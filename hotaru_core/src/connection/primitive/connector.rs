//! Outbound primitive for opening final wire streams.

use core::future::Future;

use crate::connection::{ConnStream, MaybeSend};

/// Opens outbound connections and returns the final wire stream.
pub trait Connector: Send + Sync + 'static {
    /// Stream produced by this connector.
    type Stream: ConnStream;

    /// Remote target for outbound connection.
    type Target;

    /// Connect to one remote target.
    fn connect(
        &self,
        target: Self::Target,
    ) -> impl Future<Output = std::io::Result<Self::Stream>> + MaybeSend;
}
