//! Helper functions for creating HTTP responses
//! These should eventually be in hotaru_core but are missing

use hotaru_core::http::response::HttpResponse;
use hotaru_core::http::body::HttpBody;
use hotaru_core::http::http_value::{HttpContentType, StatusCode};
use hotaru_core::Value;

/// Create a text response
pub fn text_response<S: Into<String>>(text: S) -> HttpResponse {
    let mut response = HttpResponse::default();
    response = response.content_type(HttpContentType::TextPlain());
    response.body = HttpBody::Text(text.into());
    response
}

/// Create an HTML response
pub fn html_response<S: Into<String>>(html: S) -> HttpResponse {
    let mut response = HttpResponse::default();
    response = response.content_type(HttpContentType::TextHtml());
    response.body = HttpBody::Text(html.into());
    response
}

/// Create a JSON response from an Akari Value
pub fn json_response(value: Value) -> HttpResponse {
    let mut response = HttpResponse::default();
    response = response.content_type(HttpContentType::ApplicationJson());
    response.body = HttpBody::Json(value);
    response
}

/// Create a response builder
pub fn response_builder() -> HttpResponse {
    HttpResponse::default()
}