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

/// Build a minimal HTML error page body for the given status code.
///
/// Produces a self-contained HTML document with a single `<h1>` showing the
/// status code and reason phrase (e.g. "404 Not Found").
fn html_error_body(status: &StatusCode) -> String {
    let code = status.as_u16();
    let reason = status.reason_phrase();
    format!(
        "<!DOCTYPE html>\n\
         <html>\n\
         <head><title>{code} {reason}</title></head>\n\
         <body><h1>{code} {reason}</h1></body>\n\
         </html>\n"
    )
}

/// Build a status response with an HTML body whose `<h1>` carries the
/// status code and reason phrase.
fn html_status_response(status: StatusCode) -> HttpResponse {
    let body = html_error_body(&status).into_bytes();
    response_templates::html_response(body).status(status)
}

/// Build a 404 Not Found response with an HTML `<h1>` body.
pub fn not_found_response() -> HttpResponse {
    html_status_response(StatusCode::NOT_FOUND)
}

/// Build an error response from a boxed protocol error with an HTML `<h1>`
/// body carrying the resolved status code and reason phrase.
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
    let status = if let Some(http_err) =
        (err as &dyn std::error::Error).downcast_ref::<HttpError>()
    {
        http_err.into()
    } else {
        // Fallback: generic 500 for non-HttpError protocol errors.
        StatusCode::INTERNAL_SERVER_ERROR
    };
    html_status_response(status)
}
