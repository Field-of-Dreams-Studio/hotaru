//! HTTP/1.1 protocol handler — `Http1Protocol` struct and `Protocol` trait impl.
//!
//! This is the main protocol implementation for HTTP/1.1, handling both
//! server and client roles. It is generic over the wire stream type so the
//! same logic works for TCP and TLS transports.

use std::marker::PhantomData;
use std::sync::Arc;

use hotaru_core::{
    app::common::RuntimeConfig,
    connection::{ConnStream, Outbound, TransportSpec},
    protocol::{Channel, CtxError, Protocol, ProtocolError, ProtocolFlow, ProtocolRole, RequestContext},
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
/// Generic over the concrete wire stream type so the same logic serves TCP
/// and TLS transports. Owns the per-instance `HttpSafety` baseline as an
/// `Arc` so per-connection clones are an atomic bump.
pub struct Http1Protocol<
    W: ConnStream = TcpStream,
    TS: TransportSpec<Wire = W> = DefaultHttpTransport,
> {
    role: ProtocolRole,
    safety: Arc<HttpSafety>,
    _wire: PhantomData<fn() -> W>,
    _ts: PhantomData<fn() -> TS>,
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Clone for Http1Protocol<W, TS> {
    fn clone(&self) -> Self {
        Self {
            role: self.role,
            safety: self.safety.clone(),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }
}

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Http1Protocol<W, TS> {
    /// Server role with `safety` as the per-connection baseline.
    pub fn server(safety: HttpSafety) -> Self {
        Self {
            role: ProtocolRole::Server,
            safety: Arc::new(safety),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }

    /// Client role with `safety` as the per-connection baseline.
    pub fn client(safety: HttpSafety) -> Self {
        Self {
            role: ProtocolRole::Client,
            safety: Arc::new(safety),
            _wire: PhantomData,
            _ts: PhantomData,
        }
    }

    /// Borrows this protocol's safety baseline.
    pub fn safety(&self) -> &HttpSafety {
        &self.safety
    }
}

// ============================================================================
// Protocol trait implementation
// ============================================================================

impl<W: ConnStream, TS: TransportSpec<Wire = W>> Protocol for Http1Protocol<W, TS>
where
    HttpError: From<<TS as TransportSpec>::IoError>,
{
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

    fn lit_parser<'a>(input: &'a str) -> Vec<&'a str> {
        // Mirrors UrlRoot::walk_str: empty input goes to the root
        // endpoint slot (returned via an empty segment vec), everything
        // else splits on '/' preserving empties so "/foo" walks as
        // ["", "foo"] and matches the leading Literal("") that the
        // pattern parser emits for a leading '/'.
        if input.is_empty() {
            Vec::new()
        } else {
            input.split('/').collect()
        }
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
        let safety = self.safety.clone();
        Http1Channel::new(reader, writer, meta, safety)
    }

    async fn handle(
        channel: &Self::Channel,
        runtime: Arc<RuntimeConfig>,
        root: Arc<UrlRoot<Self::Context, Self::TS>>,
    ) -> Result<ProtocolFlow, <Self::Context as RequestContext>::Error> {
        // 1. Parse one request using the channel-stored safety baseline
        //    (no per-request HashMap lookup against RuntimeConfig).
        let request = channel.parse_request(channel.safety()).await?;
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
        //    Seed ctx.safety from the protocol baseline so endpoint overrides
        //    overlay on top of it instead of falling back to defaults.
        let mut ctx = HttpContext::new_server(
            runtime.clone(),
            endpoint.clone(),
            request,
            channel.remote_addr(),
            channel.local_addr(),
            channel.safety().clone(),
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

    async fn acquire_channel(
        &self,
        _runtime: &Arc<RuntimeConfig>,
        outbound: Arc<<Self::TS as TransportSpec>::Outbound>,
    ) -> Result<Self::Channel, CtxError<Self>> {
        // Dial fresh per request. Future pooling slots in behind a
        // `self.pool` field without changing this signature.
        let wire = outbound.connect().await?;
        let (read, write, meta) = wire.split();
        let reader = BufReader::new(read);
        Ok(Http1Channel::new(reader, write, meta, self.safety.clone()))
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
            if let Some(host) = ctx.host.as_deref().filter(|h| !h.is_empty()) {
                ctx.request.meta.set_host(Some(host.to_string()));
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

    #[test]
    fn pattern_and_literal_sides_align() {
        use hotaru_core::url::tokens_to_patterns;

        let tokens = HTTP::tokenize_url("/users/<int:id>").unwrap();
        let (patterns, _names) = tokens_to_patterns(&tokens).unwrap();
        let segments = HTTP::lit_parser("/users/42");

        assert_eq!(patterns.len(), segments.len(), "leading-slash arity mismatch");
        for (pat, seg) in patterns.iter().zip(segments.iter()) {
            assert!(pat.matches(seg), "pattern {:?} did not match segment {:?}", pat, seg);
        }
    }

    #[test]
    fn root_slash_aligns() {
        use hotaru_core::url::tokens_to_patterns;

        let tokens = HTTP::tokenize_url("/").unwrap();
        let (patterns, _) = tokens_to_patterns(&tokens).unwrap();
        let segments = HTTP::lit_parser("/");

        assert_eq!(patterns.len(), segments.len());
        for (pat, seg) in patterns.iter().zip(segments.iter()) {
            assert!(pat.matches(seg));
        }
    }
}
