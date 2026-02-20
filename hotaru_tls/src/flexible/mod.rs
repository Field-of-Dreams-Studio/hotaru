//! Flexible TCP/TLS types — runtime-determined transport.
//!
//! Use these when the connection type (plain TCP vs TLS) is decided at runtime,
//! e.g. based on a URL scheme or configuration flag. For compile-time static
//! TLS, use the `tls/` module instead.

pub mod builder;
pub mod stream;

pub use builder::{Authentication, ConnectionBuilder, Protocol};
pub use stream::{FlexMeta, TcpOrTlsStream, TcpReader, TcpWriter, split_connection};
