//! One-shot HTTP/1.1 request helper.
//!
//! `send_request` runs a single request/response over any `Outbound` (TCP,
//! TLS, or anything else that produces a `ConnStream`-shaped wire). The
//! channel is opened, the request is written, the response is parsed, and
//! the channel is closed — no `Client` registration, no named outpoint.
//!
//! Lives in its own module because `Client` is the core registered-outpoint
//! abstraction in `hotaru_core::app::client`.
//!
//! ```ignore
//! // HTTP
//! let outbound = TcpOutbound::build("example.com:80".into()).await?;
//! let resp = send_request(&outbound, request, HttpSafety::default()).await?;
//!
//! // HTTPS (with the `tls` feature)
//! let outbound = TlsOutbound::build(
//!     TlsOutboundTarget::new("example.com", 443, TlsClientConfig::default()),
//! ).await?;
//! let resp = send_request(&outbound, request, HttpSafety::default()).await?;
//! ```
//!
//! The caller is responsible for setting the `Host` header on `request` —
//! the helper does not know the hostname (only the `Outbound` does).

use std::sync::Arc;

use hotaru_core::connection::{ConnStream, HotaruRead, HotaruWrite, Outbound};
use hotaru_core::protocol::Channel;

use crate::channel::Http1Channel;
use crate::channel::HttpChannel;
use crate::message::request::HttpRequest;
use crate::message::response::HttpResponse;
use crate::protocol::error::HttpError;
use crate::security::safety::HttpSafety;

/// Send one HTTP/1.1 request over a fresh wire from `outbound` and return
/// the parsed response. The channel is closed after the roundtrip.
///
/// The transport is whatever `O: Outbound` provides — plain TCP, TLS, or
/// any future transport that implements `Outbound`. The function never
/// touches scheme parsing; pick the right `Outbound` for the URL you're
/// targeting before calling.
pub async fn send_request<O>(
    outbound: &O,
    request: HttpRequest,
    safety: HttpSafety,
) -> Result<HttpResponse, HttpError>
where
    O: Outbound,
    HttpError: From<O::Error>,
    <O::Wire as ConnStream>::ReadHalf: HotaruRead<Error = std::io::Error>,
    <O::Wire as ConnStream>::WriteHalf: HotaruWrite<Error = std::io::Error>,
{
    let wire = outbound.connect().await?;
    let (read, write, meta) = wire.split();
    let channel =
        Http1Channel::<O::Wire>::new(read.into_buf(), write.into_buf_write(), meta, Arc::new(safety));

    let result = async {
        channel.send_request(request).await?;
        channel.parse_response(channel.safety()).await
    }
    .await;

    channel.close();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use hotaru_core::connection::tcp::TcpOutbound;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use crate::message::http_value::{HttpMethod, HttpVersion, StatusCode};
    use crate::message::start_line::HttpStartLine;

    async fn spawn_stub_http_server(body: &'static [u8]) -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let (mut sock, _) = match listener.accept().await {
                    Ok(x) => x,
                    Err(_) => return,
                };
                let mut buf = [0u8; 1024];
                let _ = sock.read(&mut buf).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = sock.write_all(response.as_bytes()).await;
                let _ = sock.write_all(body).await;
                let _ = sock.shutdown().await;
            }
        });
        addr
    }

    #[tokio::test]
    async fn http_roundtrip_via_tcp_outbound() {
        let addr = spawn_stub_http_server(b"pong-tcp").await;

        let outbound = TcpOutbound::build(addr.to_string()).await.unwrap();

        let mut request = HttpRequest::default();
        request.meta.start_line = HttpStartLine::new_request(
            HttpVersion::Http11,
            HttpMethod::GET,
            "/ping".to_string(),
        );
        request.meta.set_host(Some(addr.to_string()));

        let response = send_request(&outbound, request, HttpSafety::default())
            .await
            .expect("send_request");

        assert_eq!(response.meta.start_line.status_code(), StatusCode::OK);
        let body_bytes: Vec<u8> = match response.body {
            crate::message::body::HttpBody::Text(s) => s.into_bytes(),
            crate::message::body::HttpBody::Binary(b) => b,
            crate::message::body::HttpBody::Buffer { data, .. } => data,
            _ => Vec::new(),
        };
        assert_eq!(body_bytes, b"pong-tcp");
    }
}
