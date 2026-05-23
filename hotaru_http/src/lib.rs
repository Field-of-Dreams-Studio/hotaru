pub use hotaru_core::{TemplateManager, Value, app, connection, debug, executable, extensions, object, url};
pub use hotaru_core::{debug_error, debug_log, debug_trace, debug_value, debug_warn};

// ============================================================================
// Module structure organized by functional area
// ============================================================================

/// [Protocol Integration] Http1Protocol, HttpError
pub mod protocol;

/// [Channel] HttpChannel trait + Http1Channel impl (parallel to context)
pub mod channel;

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

pub use protocol::{DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol};
#[cfg(feature = "tls")]
pub use protocol::{HTTPS, Http1TlsProtocol};

// Surface the TLS transport/config pieces so users with the `tls` feature
// don't need a direct `hotaru_tls` dep.
#[cfg(feature = "tls")]
pub use hotaru_tls::{TlsClientConfig, TlsConfig, TlsOutbound, TlsOutboundTarget, TlsTransport};

pub use protocol::HttpError;

// ============================================================================
// Backward-compatible re-exports for external crates (e.g. hotaru, htmstd, h2per)
// These allow old `hotaru_http::body`, `hotaru_http::cookie`, etc. paths to work.
// ============================================================================

pub mod body {
    //! Re-exported from `message::body`
    pub use crate::message::body::*;
}

pub mod cookie {
    //! Re-exported from `util::cookie`
    pub use crate::util::cookie::*;
}

pub mod encoding {
    //! Re-exported from `util::encoding`
    pub use crate::util::encoding::*;
}

pub mod form {
    //! Re-exported from `util::form`
    pub use crate::util::form::*;
}

pub mod http_value {
    //! Re-exported from `message::http_value`
    pub use crate::message::http_value::*;
}

pub mod meta {
    //! Re-exported from `message::meta`
    pub use crate::message::meta::*;
}

pub mod request {
    //! Re-exported from `message::request`
    pub use crate::message::request::*;
}

pub mod response {
    //! Re-exported from `message::response`
    pub use crate::message::response::*;
}

pub mod safety {
    //! Re-exported from `security::safety`
    pub use crate::security::safety::*;
}

pub mod traits {
    //! Re-exported from `protocol::traits`
    pub use crate::protocol::traits::*;
}
