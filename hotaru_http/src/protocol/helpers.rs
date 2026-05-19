//! Helper functions for the HTTP/1 protocol handle loop.

use hotaru_core::protocol::ProtocolError;

use crate::message::http_value::StatusCode;
use crate::message::request::HttpRequest;
use crate::message::response::{HttpResponse, response_templates};
use crate::protocol::error::HttpError;

/// Check whether the HTTP request indicates keep-alive should be used.
pub fn is_keep_alive(request: &HttpRequest) -> bool {
    if let Some(connection) = request.meta.header.get("connection") {
        connection.as_str().to_lowercase() != "close"
    } else {
        // HTTP/1.1 defaults to keep-alive
        true
    }
}

/// Build a 404 Not Found response.
pub fn not_found_response() -> HttpResponse {
    response_templates::return_status(StatusCode::NOT_FOUND)
}

/// Build an error response from a boxed protocol error.
///
/// Maps each `HttpError` variant to the most appropriate HTTP status code:
///
/// | HttpError Variant | HTTP Status Code |
/// |---|---|
/// | `Io` | 500 Internal Server Error |
/// | `Connection` | 502 Bad Gateway |
/// | `ParseError` | 400 Bad Request |
/// | `InvalidHeader` | 400 Bad Request |
/// | `InvalidUri` | 400 Bad Request |
/// | `ChunkError` | 400 Bad Request |
/// | `PayloadTooLarge` | 413 Payload Too Large |
/// | `MethodNotAllowed` | 405 Method Not Allowed |
/// | `UnsupportedMediaType` | 415 Unsupported Media Type |
/// | `HeaderTooLarge` | 431 Request Header Fields Too Large |
/// | `TooManyHeaders` | 431 Request Header Fields Too Large |
/// | `HeaderLineTooLong` | 431 Request Header Fields Too Large |
/// | `Status(code)` | The wrapped status code |
/// | `NoRoute` | 404 Not Found |
/// | `Timeout` | 408 Request Timeout |
/// | `VersionNotSupported` | 505 HTTP Version Not Supported |
/// | `ProtocolViolation` | 400 Bad Request |
/// | `Other` | 500 Internal Server Error |
pub fn error_response_from(err: &dyn ProtocolError) -> HttpResponse {
    // Try to downcast to HttpError for fine-grained status mapping.
    // ProtocolError: std::error::Error + Send + Sync + 'static, so we can
    // downcast through the std::error::Error vtable.
    if let Some(http_err) = (err as &dyn std::error::Error).downcast_ref::<HttpError>() {
        let status: StatusCode = http_err.into();
        response_templates::return_status(status)
    } else {
        // Fallback: generic 500 for non-HttpError protocol errors.
        response_templates::return_status(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
