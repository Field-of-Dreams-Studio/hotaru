//! HTTP/1.1 protocol handler — `Http1Protocol` struct and `Protocol` trait impl.
//!
//! This is the main protocol implementation for HTTP/1.1, handling both
//! server and client roles. It is generic over the wire stream type so the
//! same logic works for TCP and TLS transports.

use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use hotaru_core::{
    app::common::RuntimeConfig,
    connection::{ConnStream, TransportSpec},
    protocol::{Channel, Protocol, ProtocolError, ProtocolFlow, ProtocolRole, RequestContext},
    url::UrlRoot,
};
use tokio::io::BufReader;
use tokio::net::TcpStream;

use crate::{
    channel::{Http1Channel, HttpChannel},
    context::HttpContext,
    protocol::{
        error::HttpError,
        helpers::{error_response_from, is_keep_alive, is_response_keep_alive, not_found_response},
    },
    security::safety::HttpSafety,
};

// ============================================================================
// Type Aliases
// ============================================================================

/// Default transport spec used by HTTP when callers don't specify one.
pub type DefaultHttpTransport = hotaru_core::connection::tcp::TcpTransport;

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
// Http1Protocol
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
    role: ProtocolRole,
    _wire: PhantomData<fn() -> W>,
    _ts: PhantomData<fn() -> TS>,
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Clone for Http1Protocol<W, TS> {
    fn clone(&self) -> Self {
        Self {
            role: self.role,
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Http1Protocol<W, TS> {
    /// Creates a new HTTP/1.1 protocol handler for server role.
    ///
    /// The `_safety` parameter is accepted for API compatibility; the live
    /// `HttpSafety` used per request comes from `RuntimeConfig`.
    pub fn server(_safety: HttpSafety) -> Self {
        Self {
            role: ProtocolRole::Server,
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }

    /// Creates a new HTTP/1.1 protocol handler for client role.
    ///
    /// See [`server`](Self::server) regarding `_safety`.
    pub fn client(_safety: HttpSafety) -> Self {
        Self {
            role: ProtocolRole::Client,
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }
}

// ============================================================================
// Protocol trait implementation
// ============================================================================

#[async_trait]
impl<W: ConnStream, TS: TransportSpec<Wire = W>> Protocol for Http1Protocol<W, TS> {
    type Wire = W;
    type TS = TS;
    type Channel = Http1Channel<W>;
    type Stream = ();
    type Message = ();
    type Context = HttpContext<TS>;

    fn name(&self) -> &'static str {
        "http"
    }

    fn role(&self) -> ProtocolRole {
        self.role
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

    fn open_channel(
        self,
        reader: BufReader<<<Self::TS as TransportSpec>::Wire as ConnStream>::ReadHalf>,
        writer: <<Self::TS as TransportSpec>::Wire as ConnStream>::WriteHalf,
        meta: <<Self::TS as TransportSpec>::Wire as ConnStream>::Meta,
    ) -> Self::Channel {
        Http1Channel::new(reader, writer, meta)
    }

    async fn handle(
        channel: &Self::Channel,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::TS>>,
    ) -> Result<ProtocolFlow, <Self::Context as RequestContext>::Error> {
        let safety = runtime.get_config::<HttpSafety>().unwrap_or_default();

        // 1. Parse one request.
        let request = channel.parse_request(&safety).await?;
        let keep_alive = is_keep_alive(&request);

        // 2. Walk URL tree.
        let path = request.meta.path();
        let endpoint = match root.walk_str(&path).await {
            Some(node) => node,
            None => {
                // No route: send 404 and decide based on keep-alive.
                channel.send_response(not_found_response()).await?;
                return Ok(if keep_alive { ProtocolFlow::Continue } else { ProtocolFlow::Close });
            }
        };

        // 3. Build context, run chain. Addresses come from the channel's meta.
        let mut ctx = HttpContext::new_server(
            runtime.clone(),
            endpoint.clone(),
            request,
            channel.remote_addr(),
            channel.local_addr(),
        );
        ctx.install_channel(channel.clone());

        match endpoint.run(ctx).await {
            Ok(ctx) => {
                channel.send_response(ctx.response).await?;
                Ok(if keep_alive { ProtocolFlow::Continue } else { ProtocolFlow::Close })
            }
            Err(err) if err.can_continue() => {
                // Recoverable: map error to a response and keep going.
                channel.send_response(error_response_from(&err)).await?;
                Ok(if keep_alive { ProtocolFlow::Continue } else { ProtocolFlow::Close })
            }
            Err(_) => Ok(ProtocolFlow::Close),
        }
    }

    async fn send(
        mut ctx: Self::Context,
    ) -> Result<Self::Context, <Self::Context as RequestContext>::Error> {
        let channel = ctx
            .channel()
            .cloned()
            .ok_or_else(|| HttpError::ProtocolViolation("outpoint channel is not installed".to_string()))?;

        let safety = ctx.safety.clone();

        if ctx.request.meta.get_host().is_none() {
            if let Some(host) = ctx.host.clone() {
                ctx.request.meta.set_host(Some(host));
            }
        }

        let request = std::mem::take(&mut ctx.request);
        channel.send_request(request).await?;
        ctx.response = channel.parse_response(&safety).await?;

        if !is_response_keep_alive(&ctx.response) {
            channel.close();
        }
        Ok(ctx)
    }

    fn install_channel(ctx: &mut Self::Context, channel: Self::Channel) {
        ctx.install_channel(channel);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::http_value::StatusCode;
    use crate::message::meta::HeaderValue;
    use crate::message::request::HttpRequest;

    #[test]
    fn test_http1_detection() {
        assert!(HTTP::detect(b"GET / HTTP/1.1\r\n"));
        assert!(HTTP::detect(b"POST /api HTTP/1.1\r\n"));
        assert!(HTTP::detect(b"PUT /resource HTTP/1.1\r\n"));
        assert!(!HTTP::detect(b"INVALID REQUEST\r\n"));
        assert!(!HTTP::detect(b""));
    }

    #[test]
    fn test_is_keep_alive() {
        let mut request = HttpRequest::default();
        // No Connection header → HTTP/1.1 default keep-alive
        assert!(is_keep_alive(&request));

        // Connection: close
        request.meta.header.insert("connection".to_string(), HeaderValue::Single("close".to_string()));
        assert!(!is_keep_alive(&request));

        // Connection: keep-alive
        request.meta.header.insert("connection".to_string(), HeaderValue::Single("keep-alive".to_string()));
        assert!(is_keep_alive(&request));
    }

    #[test]
    fn test_not_found_response() {
        let resp = not_found_response();
        assert_eq!(resp.meta.start_line.status_code(), StatusCode::NOT_FOUND);
    }
}
