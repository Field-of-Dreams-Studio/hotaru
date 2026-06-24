use super::{ConnStream, Inbound, Outbound};

/// Static transport policy for one runtime.
pub trait TransportSpec: Send + Sync + 'static {
    /// Wire stream used by protocols.
    type Wire: ConnStream;

    /// Transport-level IO error. `std::io::Error` for std-flavoured
    /// transports; embedded transports pick their own. Both `Inbound`
    /// and `Outbound` are pinned to this type so server and client
    /// error surfaces stay aligned.
    type IoError: core::error::Error + Send + Sync + 'static;

    /// App-facing inbound runtime.
    type Inbound: Inbound<Wire = Self::Wire, Error = Self::IoError>;

    /// App-facing outbound runtime.
    type Outbound: Outbound<Wire = Self::Wire, Error = Self::IoError>;

    /// Default inbound bind target, if no config is needed.
    fn default_inbound() -> Option<<Self::Inbound as Inbound>::BindTarget> {
        None
    }

    /// Default outbound connect target, if one is appropriate.
    fn default_outbound() -> Option<<Self::Outbound as Outbound>::ConnectTarget> {
        None
    }
}
