pub mod traits;
pub mod error;

pub use error::HttpError;
pub use traits::{
    DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol, HttpTransport,
};
#[cfg(feature = "tls")]
pub use traits::{HTTPS, Http1TlsProtocol};
