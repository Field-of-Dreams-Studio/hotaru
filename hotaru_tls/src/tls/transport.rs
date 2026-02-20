//! TlsTransport — static TransportSpec for TLS connections.

use hotaru_core::connection::TransportSpec;

use super::{accepter::TlsAccepter, connector::TlsConnector, stream::TlsStream};

/// Transport policy for TLS-encrypted connections.
///
/// Ties `TlsStream` as the wire type with `TlsAccepter` (server side)
/// and `TlsConnector` (client side).
///
/// Note: `TlsAccepter` and `TlsConnector` both require runtime config
/// (certificates/keys), so `default_accepter()` and `default_connector()`
/// return `None`. Build them via `TlsAccepter::new(config)` / `TlsConnector::new(config)`.
pub struct TlsTransport;

impl TransportSpec for TlsTransport {
    type Wire = TlsStream;
    type Accepter = TlsAccepter;
    type Connector = TlsConnector;
}
