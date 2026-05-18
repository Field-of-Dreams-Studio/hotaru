//! Re-exports from the protocol sub-modules for backward compatibility.
//!
//! This file exists so that existing code importing from `hotaru_http::traits`
//! or `hotaru_http::protocol::traits` continues to work after the structs were
//! split into separate files (`transport`, `channel`, `helpers`, `protocol_impl`).

pub use super::{
    channel::Http1Channel,
    helpers::{error_response_from, is_keep_alive, not_found_response},
    http_channel::HttpChannel,
    protocol_impl::{
        DefaultHttpTransport, HTTP, Http1Protocol, Http1TcpProtocol,
    },
    transport::HttpTransport,
};
#[cfg(feature = "tls")]
pub use super::protocol_impl::{HTTPS, Http1TlsProtocol};
