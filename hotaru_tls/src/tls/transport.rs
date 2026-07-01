//! TlsTransport — static TransportSpec for TLS connections.

use hotaru_core::connection::TransportSpec;

use super::{
    runtime::{TlsInbound, TlsOutbound},
    stream::TlsStream,
};

/// Transport policy for TLS-encrypted connections.
///
/// Ties `TlsStream` as the wire type with TLS inbound/outbound runtimes.
///
/// TLS requires certificate config, so no default bind/connect target is
/// provided. Build `TlsInboundTarget` / `TlsOutboundTarget` explicitly.
pub struct TlsTransport;

impl TransportSpec for TlsTransport {
    type Wire = TlsStream;
    type IoError = std::io::Error;
    type Inbound = TlsInbound;
    type Outbound = TlsOutbound;
}
