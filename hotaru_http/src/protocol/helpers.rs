//! Helper functions for the HTTP/1 protocol handle loop.

use hotaru_core::protocol::ProtocolError;

use crate::message::http_value::StatusCode;
use crate::message::request::HttpRequest;
use crate::message::response::{HttpResponse, response_templates};

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
pub fn error_response_from(_err: &dyn ProtocolError) -> HttpResponse {
    // Generic 500 Internal Server Error for recoverable errors.
    response_templates::return_status(StatusCode::INTERNAL_SERVER_ERROR)
}
