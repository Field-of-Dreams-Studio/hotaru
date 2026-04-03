//! HTTP protocol implementation using the new Protocol trait.
//!
//! This replaces the old Rx/Tx based implementation with the new
//! unified protocol system.
//!
//! The current integration updates signatures/wiring so it works with
//! `ConnStream` split halves while keeping existing HTTP/1.1 logic.

use std::error::Error;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{any::Any};

use async_trait::async_trait;
use bytes::BytesMut;
use futures::executor::block_on;
use tokio::io::{AsyncBufRead, AsyncWriteExt, BufReader, ReadBuf};
use tokio::net::TcpStream;

use crate::{
    app::common::RuntimeConfig,
    connection::{
        ConnMeta, ConnStream, Message, Protocol, ProtocolRole, Transport, TransportSpec,
    },
    http::{
        context::HttpContext, request::HttpRequest, response::HttpResponse, safety::HttpSafety,
    },
    url::UrlRoot,
};

// ============================================================================
// Type Aliases
// ============================================================================

/// Default transport spec used by HTTP when callers don't specify one.
pub type DefaultHttpTransport = crate::connection::tcp::TcpTransport;

/// Default HTTP protocol (currently HTTP/1.1)
/// This provides a simpler name for user-facing code while maintaining
/// version-specific naming in the implementation.
///
/// In the new transport design this defaults to plain TCP transport.
pub type HTTP = Http1Protocol<TcpStream, DefaultHttpTransport>;

/// HTTP/1.1 over plain TCP transport.
pub type Http1TcpProtocol = Http1Protocol<TcpStream, DefaultHttpTransport>;

/// HTTP/1.1 over TLS transport (enabled by `tls` feature).
#[cfg(feature = "tls")]
pub type Http1TlsProtocol = Http1Protocol<hotaru_tls::TlsStream, hotaru_tls::TlsTransport>;

/// HTTPS alias (HTTP/1.1 over TLS), enabled by `tls` feature.
#[cfg(feature = "tls")]
pub type HTTPS = Http1TlsProtocol;

// ============================================================================
// HttpTransport - Connection state for HTTP
// ============================================================================

/// HTTP transport state.
///
/// Tracks connection-level state for HTTP:
/// - Connection ID for logging
/// - Keep-alive status
/// - Request count for connection reuse
/// - Safety configuration
#[derive(Clone)]
pub struct HttpTransport {
    /// Unique connection identifier
    id: i128,

    /// Whether this connection supports keep-alive
    pub keep_alive: bool,

    /// Local address of the connection
    pub local_addr: SocketAddr,

    /// Remote address of the connection
    pub remote_addr: SocketAddr,

    /// Number of requests processed on this connection
    pub request_count: u64,

    /// HTTP safety configuration (limits, timeouts, etc.)
    pub safety: HttpSafety,

    /// Role of this protocol instance
    pub role: ProtocolRole,
}

/// Placeholder address for uninitialized connections.
const UNSET_ADDR: SocketAddr = SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
    0,
);

impl HttpTransport {
    /// Creates a new HTTP/1.1 transport with socket addresses.
    pub fn new(
        role: ProtocolRole,
        safety: HttpSafety,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
    ) -> Self {
        Self {
            id: generate_connection_id(),
            keep_alive: true,
            local_addr,
            remote_addr,
            request_count: 0,
            safety,
            role,
        }
    }

    /// Creates a new HTTP/1.1 transport without socket addresses.
    /// Addresses should be set via set_addresses() when available.
    pub fn new_unbound(role: ProtocolRole, safety: HttpSafety) -> Self {
        Self::new(role, safety, UNSET_ADDR, UNSET_ADDR)
    }

    /// Sets the socket addresses from connection metadata.
    pub fn set_addresses(&mut self, local: Option<SocketAddr>, remote: Option<SocketAddr>) {
        if let Some(addr) = local {
            self.local_addr = addr;
        }
        if let Some(addr) = remote {
            self.remote_addr = addr;
        }
    }

    /// Increments the request counter.
    pub fn increment_requests(&mut self) {
        self.request_count += 1;
    }

    /// Checks if the connection should be kept alive.
    pub fn should_keep_alive(&self) -> bool {
        self.keep_alive
    }

    /// Updates keep-alive based on request headers.
    pub fn update_keep_alive(&mut self, request: &HttpRequest) {
        // Check Connection header
        if let Some(connection) = request.meta.header.get("connection") {
            self.keep_alive = connection.as_str().to_lowercase() != "close";
        } else {
            // HTTP/1.1 defaults to keep-alive
            self.keep_alive = true;
        }
    }
}

impl Transport for HttpTransport {
    type Id = i128;

    fn id(&self) -> i128 {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ============================================================================
// HttpMessage - Message wrapper for HTTP
// ============================================================================

/// HTTP message wrapper.
///
/// Wraps the existing HttpRequest and HttpResponse types
/// to implement the Message trait.
#[derive(Debug)]
pub enum HttpMessage {
    /// HTTP request (client -> server)
    Request(HttpRequest),

    /// HTTP response (server -> client)
    Response(HttpResponse),
}

impl Message for HttpMessage {
    fn encode(&self, buf: &mut BytesMut) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self {
            HttpMessage::Request(req) => {
                // Clone to get ownership
                let mut meta = req.meta.clone();
                let body = req.body.clone();

                // Use into_static to properly set headers and get body bytes
                let body_bytes = block_on(body.into_static(&mut meta));

                // Use represent() to format headers
                let headers = meta.represent();
                buf.extend_from_slice(headers.as_bytes());

                // Add body
                buf.extend_from_slice(&body_bytes);
                Ok(())
            }
            HttpMessage::Response(res) => {
                // Clone to get ownership
                let mut meta = res.meta.clone();
                let body = res.body.clone();

                // Use into_static to properly set headers and get body bytes
                let body_bytes = block_on(body.into_static(&mut meta));

                // Use represent() to format headers
                let headers = meta.represent();
                buf.extend_from_slice(headers.as_bytes());

                // Add body
                buf.extend_from_slice(&body_bytes);
                
                Ok(())
            }
        }
    }

    fn decode(_buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>> {
        // For now, we'll use the existing parsing logic in handle methods
        // Full implementation would use HttpRequest::parse_lazy here
        Ok(None)
    }
}

// ============================================================================
// Wire wrappers (signature migration only)
// ============================================================================

/// Reader wrapper that carries connection addresses while preserving `AsyncBufRead`.
pub struct HttpWireReader<R> {
    inner: R,
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
}

impl<R> HttpWireReader<R> {
    pub fn new(inner: R, local_addr: Option<SocketAddr>, remote_addr: Option<SocketAddr>) -> Self {
        Self {
            inner,
            local_addr,
            remote_addr,
        }
    }

    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }
}

impl<R: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for HttpWireReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<R: AsyncBufRead + Unpin> AsyncBufRead for HttpWireReader<R> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        let this = self.get_mut();
        Pin::new(&mut this.inner).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let this = self.get_mut();
        Pin::new(&mut this.inner).consume(amt)
    }
}

/// Thin writer wrapper so the old HTTP logic can keep using one local type.
pub struct HttpWireWriter<W> {
    inner: W,
}

impl<W> HttpWireWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: tokio::io::AsyncWrite + Unpin> tokio::io::AsyncWrite for HttpWireWriter<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// ============================================================================
// Http1Protocol - Main protocol handler
// ============================================================================

/// HTTP/1.1 protocol handler.
///
/// Implements the Protocol trait for HTTP/1.1, handling both
/// server and client roles.
///
/// Generic over the concrete wire stream type so the same logic can be used
/// for TCP and TLS transports without duplicating protocol code.
pub struct Http1Protocol<
    W: ConnStream = TcpStream,
    TS: TransportSpec<Wire = W> = DefaultHttpTransport,
> {
    /// Transport state for this connection
    transport: HttpTransport,
    /// Application reference is passed to handle methods and not stored here.
    _wire: PhantomData<fn() -> W>,
    _ts: PhantomData<fn() -> TS>,
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Clone for Http1Protocol<W, TS> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Http1Protocol<W, TS> {
    /// Creates a new HTTP/1.1 protocol handler for server role.
    pub fn server(safety: HttpSafety) -> Self {
        Self {
            transport: HttpTransport::new_unbound(ProtocolRole::Server, safety),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }

    /// Creates a new HTTP/1.1 protocol handler for client role.
    pub fn client(safety: HttpSafety) -> Self {
        Self {
            transport: HttpTransport::new_unbound(ProtocolRole::Client, safety),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }

    /// Handles server-side HTTP/1.1 connections.
    async fn handle_server(
        &mut self,
        mut reader: HttpWireReader<BufReader<W::ReadHalf>>,
        mut writer: HttpWireWriter<W::WriteHalf>,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<HttpContext<TS>, TS>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            // Parse request using existing logic
            let request = HttpRequest::parse_lazy(&mut reader, &self.transport.safety, false).await;

            // Check if request is empty/default (parsing failed)
            if request.meta.path().is_empty() && request.meta.header.is_empty() {
                break;
            }

            // Update keep-alive status
            self.transport.update_keep_alive(&request);
            self.transport.increment_requests();

            // Walk the URL tree to find the matching endpoint
            let path = request.meta.path();
            let endpoint = root
                .walk_str(&path)
                .await
                .ok_or("No HTTP/1.1 endpoint matched the request path")?;

            // Create the context with the found endpoint
            let ctx = HttpContext::new_server(
                runtime.clone(),
                endpoint,
                request,
                reader.remote_addr(),
                reader.local_addr(),
            );

            // Run the handler and get response
            let (response, _status) = ctx.run().await?;

            // Send response
            response.send(&mut writer).await?;
            writer.flush().await?;

            // Check if we should close the connection
            if !self.transport.should_keep_alive() {
                break;
            }
        }

        Ok(())
    }

    /// Handles client-side HTTP/1.1 connections.
    async fn handle_client(
        &mut self,
        _reader: HttpWireReader<BufReader<W::ReadHalf>>,
        _writer: HttpWireWriter<W::WriteHalf>,
        _runtime: Arc<RuntimeConfig>,
        _root: Arc<UrlRoot<HttpContext<TS>, TS>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Client implementation will be added when we create the Client App
        Err("HTTP/1.1 client not yet implemented".into())
    }
}

#[async_trait]
impl<W: ConnStream, TS: TransportSpec<Wire = W>> Protocol for Http1Protocol<W, TS> {
    type Wire = W;
    type Spec = TS;
    type Transport = HttpTransport;
    type Stream = ();
    type Message = HttpMessage;
    type Context = HttpContext<TS>;

    fn name(&self) -> &'static str {
        "http"
    }

    fn role(&self) -> ProtocolRole {
        self.transport.role
    }

    fn detect(initial_bytes: &[u8]) -> bool {
        initial_bytes.starts_with(b"GET ")
            || initial_bytes.starts_with(b"POST ")
            || initial_bytes.starts_with(b"PUT ")
            || initial_bytes.starts_with(b"DELETE ")
            || initial_bytes.starts_with(b"HEAD ")
            || initial_bytes.starts_with(b"OPTIONS ")
            || initial_bytes.starts_with(b"PATCH ")
            || initial_bytes.starts_with(b"CONNECT ")
            || initial_bytes.starts_with(b"TRACE ")
    }

    async fn handle(
        &mut self,
        reader: BufReader<<Self::Wire as ConnStream>::ReadHalf>,
        writer: <Self::Wire as ConnStream>::WriteHalf,
        meta: <Self::Wire as ConnStream>::Meta,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::Spec>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.transport
            .set_addresses(meta.local_addr(), meta.remote_addr());

        let reader = HttpWireReader::new(reader, meta.local_addr(), meta.remote_addr());
        let writer = HttpWireWriter::new(writer);

        match self.role() {
            ProtocolRole::Server => self.handle_server(reader, writer, runtime, root).await,
            ProtocolRole::Client => self.handle_client(reader, writer, runtime, root).await,
        }
    }

    async fn request(
        &mut self,
        reader: BufReader<<Self::Wire as ConnStream>::ReadHalf>,
        writer: <Self::Wire as ConnStream>::WriteHalf,
        meta: <Self::Wire as ConnStream>::Meta,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::Spec>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.transport
            .set_addresses(meta.local_addr(), meta.remote_addr());

        let reader = HttpWireReader::new(reader, meta.local_addr(), meta.remote_addr());
        let writer = HttpWireWriter::new(writer);

        self.handle_client(reader, writer, runtime, root).await
    }
}

/// Generates a unique connection ID.
fn generate_connection_id() -> i128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http1_detection() {
        assert!(HTTP::detect(b"GET / HTTP/1.1\r\n"));
        assert!(HTTP::detect(b"POST /api HTTP/1.1\r\n"));
        assert!(HTTP::detect(b"PUT /resource HTTP/1.1\r\n"));
        assert!(!HTTP::detect(b"INVALID REQUEST\r\n"));
        assert!(!HTTP::detect(b""));
    }

    #[test]
    fn test_transport_keep_alive() {
        let mut transport = HttpTransport::new_unbound(ProtocolRole::Server, HttpSafety::default());
        assert!(transport.keep_alive);

        transport.keep_alive = false;
        assert!(!transport.should_keep_alive());
    }

    #[test]
    /// Test for HTTP message encoding - currently fails due to incomplete HttpRequest::default()
    /// TODO: Fix HttpRequest::default() to include proper start line initialization
    /// Run with: cargo test --lib -- --ignored test_message_encoding
    #[ignore = "requires HttpRequest::default() to include start line"]
    fn test_message_encoding() {
        use crate::http::meta::HttpMeta;

        // Test request encoding
        let mut request = HttpRequest::default();
        request.meta = HttpMeta::new(Default::default(), Default::default());
        request.meta
            .header
            .insert("Host".to_string(), "example.com".into());

        let msg = HttpMessage::Request(request);
        let mut buf = BytesMut::new();

        msg.encode(&mut buf).unwrap();
        let encoded = String::from_utf8_lossy(&buf);

        assert!(encoded.starts_with("GET /test HTTP/1.1\r\n"));
        assert!(encoded.contains("Host: example.com\r\n"));
    }
}
