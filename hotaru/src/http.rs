//! HTTP-specific re-exports for standard Hotaru HTTP implementation
//!
//! Use this module when working with the standard HTTP protocol:
//! ```rust
//! use hotaru::http::*;
//! ```

// HTTP protocol traits and transports
pub use hotaru_http::traits::{
    DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol,
};

// HTTPS + TLS types (gated by the `https` feature on the umbrella crate).
#[cfg(feature = "https")]
pub use hotaru_http::{
    HTTPS, Http1TlsProtocol, TlsClientConfig, TlsConfig, TlsOutbound, TlsOutboundTarget,
    TlsTransport,
};

// Request / response / context
pub use hotaru_http::context::{Executable, HttpContext};
pub use hotaru_http::context::HttpReqCtx;
/// Response-context alias kept for source compatibility with old
/// Starberry-era code. Use [`HttpContext`] directly in new code.
pub type HttpResCtx = HttpContext;
pub use hotaru_http::request::HttpRequest;
pub use hotaru_http::response::HttpResponse;

// HTTP types
pub use hotaru_http::body::*;
pub use hotaru_http::cookie::*;
pub use hotaru_http::encoding::*;
pub use hotaru_http::form::*;
pub use hotaru_http::http_value::HttpMethod::*;
pub use hotaru_http::http_value::*;
pub use hotaru_http::meta::*;
pub use hotaru_http::safety::HttpSafety;
pub use hotaru_http::start_line::*;
pub use hotaru_http::send_request;

// Request and response templates
pub use hotaru_http::request::request_templates;
pub use hotaru_http::response::response_templates;
pub use hotaru_http::request::request_templates::*;
pub use hotaru_http::response::response_templates::*;
