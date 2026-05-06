use super::{ConnStream, Inbound, Outbound};

/// Static transport policy for one runtime.
pub trait TransportSpec: Send + Sync + 'static {
    /// Wire stream used by protocols.
    type Wire: ConnStream;

    /// App-facing inbound runtime.
    type Inbound: Inbound<Wire = Self::Wire>;

    /// App-facing outbound runtime.
    type Outbound: Outbound<Wire = Self::Wire>;

    /// Default inbound bind target, if no config is needed.
    fn default_inbound() -> Option<<Self::Inbound as Inbound>::BindTarget> {
        None
    }

    /// Default outbound connect target, if one is appropriate.
    fn default_outbound() -> Option<<Self::Outbound as Outbound>::ConnectTarget> {
        None
    }
}
