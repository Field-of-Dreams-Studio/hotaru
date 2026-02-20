pub use hotaru_core::{
    app, connection, debug, extensions, url, object, TemplateManager, Value,
};
pub use hotaru_core::{debug_error, debug_log, debug_trace, debug_warn, debug_value};

pub mod request;
pub mod body;
pub mod context;
pub mod cookie;
pub mod encoding;
pub mod form;
pub mod meta;
pub mod http_value;
pub mod response;
pub mod net;
pub mod start_line;
pub mod safety;
pub mod traits;  // Protocol trait implementations for HTTP/1.1

#[cfg(test)]
pub mod test;  // Security tests for HTTP parsing

/// Compatibility namespace so legacy `crate::http::...` imports keep working
/// after moving HTTP into this standalone crate.
pub mod http {
    pub use crate::body;
    pub use crate::context;
    pub use crate::cookie;
    pub use crate::encoding;
    pub use crate::form;
    pub use crate::http_value;
    pub use crate::meta;
    pub use crate::net;
    pub use crate::request;
    pub use crate::response;
    pub use crate::safety;
    pub use crate::start_line;
    pub use crate::traits;
}

pub use traits::{
    DefaultHttpTransport, Http1Protocol, Http1TcpProtocol, HTTP, HttpTransport,
    HttpMessage,
};
#[cfg(feature = "tls")]
pub use traits::{Http1TlsProtocol, HTTPS};
