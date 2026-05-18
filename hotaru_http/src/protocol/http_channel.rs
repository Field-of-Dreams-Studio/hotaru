//! HTTP-level channel trait — the I/O surface every HTTP version must provide.
//!
//! `HttpChannel` extends the framework-level [`Channel`] trait with HTTP-specific
//! I/O methods (parse request/response, send request/response). Every HTTP
//! version (HTTP/1.1, HTTP/2, HTTP/3) implements this trait so that the
//! protocol-level `handle` / `send` logic can be written generically.

use hotaru_core::protocol::Channel;

use crate::message::request::HttpRequest;
use crate::message::response::HttpResponse;
use crate::protocol::error::HttpError;
use crate::security::safety::HttpSafety;

/// HTTP-level channel — the I/O surface every HTTP version must provide.
///
/// This trait sits between the framework [`Channel`] trait (which only knows
/// about open/close) and the concrete HTTP version implementations. It
/// defines the four fundamental HTTP I/O operations:
///
/// - Parse an incoming request from the wire
/// - Send a response on the wire
/// - Send a request on the wire (client-side)
/// - Parse an incoming response from the wire (client-side)
pub trait HttpChannel: Channel {
    /// Parse one HTTP request from the channel's reader.
    ///
    /// On EOF / malformed input, implementations should flip the channel
    /// closed and return an [`HttpError::Io`] with `UnexpectedEof`.
    async fn parse_request(&self, safety: &HttpSafety) -> Result<HttpRequest, HttpError>;

    /// Send an HTTP response on the channel's writer.
    async fn send_response(&self, response: HttpResponse) -> Result<(), HttpError>;

    /// Send an HTTP request on the channel's writer (client-side).
    async fn send_request(&self, request: HttpRequest) -> Result<(), HttpError>;

    /// Parse one HTTP response from the channel's reader (client-side).
    async fn parse_response(&self, safety: &HttpSafety) -> Result<HttpResponse, HttpError>;
}
