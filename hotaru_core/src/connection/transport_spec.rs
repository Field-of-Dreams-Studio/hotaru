use super::{Accepter, ConnStream, Connector};

/// Static transport policy for one application/runtime.
///
/// `TransportSpec` ties together the wire stream type and the concrete
/// inbound/outbound transport implementations used to create that stream.
pub trait TransportSpec: Send + Sync + 'static {
    /// Wire-level stream used by protocols in this runtime.
    type Wire: ConnStream;

    /// Inbound upgrader (server side).
    type Accepter: Accepter<Stream = Self::Wire>;

    /// Outbound connector (client side).
    type Connector: Connector<Stream = Self::Wire>;

    /// Optional default inbound upgrader for this transport.
    ///
    /// Return `None` when the transport requires explicit runtime config.
    fn default_accepter() -> Option<Self::Accepter> {
        None
    }

    /// Optional default outbound connector for this transport.
    ///
    /// Return `None` when the transport requires explicit runtime config.
    fn default_connector() -> Option<Self::Connector> {
        None
    }
}
