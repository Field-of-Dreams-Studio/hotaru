pub use hotaru_core::{TemplateManager, Value, app, connection, debug, executable, extensions, object, url};
pub use hotaru_core::{debug_error, debug_log, debug_trace, debug_value, debug_warn};

// ============================================================================
// Module structure organized by functional area
// ============================================================================

/// [Protocol Integration] Http1Protocol, Http1Channel, HttpTransport, HttpError
pub mod protocol;

/// [Runtime Context] HttpContext, parse_lazy(), send()
pub mod context;

/// [Message Model] HttpRequest, HttpResponse, HttpBody, HttpMeta, HttpStartLine, types
pub mod message;

/// [Security] HttpSafety
pub mod security;

/// [Utilities] Cookie, encoding, form, security tests
pub mod util;

// ============================================================================
// Re-exports for backward compatibility
// ============================================================================

pub use protocol::{
    DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol, HttpTransport,
};
#[cfg(feature = "tls")]
pub use protocol::{HTTPS, Http1TlsProtocol};

pub use protocol::HttpError;

/// Compatibility namespace so legacy `crate::http::...` imports keep working
/// after moving HTTP into this standalone crate.
pub mod http {
    pub use crate::context;
    pub use crate::message::{body, http_value, meta, request, response, start_line};
    pub use crate::protocol;
    pub use crate::security;
    pub use crate::util::{cookie, encoding, form};
}
