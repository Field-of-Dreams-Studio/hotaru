pub use hotaru_core::{TemplateManager, Value, app, connection, debug, executable, extensions, object, protocol, url};
pub use hotaru_core::{debug_error, debug_log, debug_trace, debug_value, debug_warn};

pub mod body;
pub mod context;
pub mod cookie;
pub mod encoding;
pub mod form;
pub mod http_value;
pub mod meta;
pub mod net;
pub mod request;
pub mod response;
pub mod safety;
pub mod start_line;
pub mod traits; // Protocol trait implementations for HTTP/1.1
pub mod error;

#[cfg(test)]
pub mod test; // Security tests for HTTP parsing

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

pub use error::HttpError;
pub use traits::{
    DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol, HttpTransport,
};
#[cfg(feature = "tls")]
pub use traits::{HTTPS, Http1TlsProtocol};
