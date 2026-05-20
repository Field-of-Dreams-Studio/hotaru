//! Re-exports from the protocol sub-modules for backward compatibility.
//!
//! Existing code importing from `hotaru_http::traits` or
//! `hotaru_http::protocol::traits` continues to work after the structs were
//! split into separate files (`helpers`, `protocol_impl`) and the channel
//! layer was moved to the sibling `channel` module.
//!
//! `HttpTransport` was removed; addresses now live on `Http1Channel` via
//! `HttpChannel::local_addr` / `remote_addr`.

pub use super::{
    helpers::{error_response_from, is_keep_alive, not_found_response},
    protocol_impl::{
        DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol,
    },
};
pub use crate::channel::{Http1Channel, HttpChannel};
#[cfg(feature = "tls")]
pub use super::protocol_impl::{HTTPS, Http1TlsProtocol};
