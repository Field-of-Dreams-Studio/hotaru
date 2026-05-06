//! Outbound primitive for opening final wire streams.

use async_trait::async_trait;

use crate::connection::ConnStream;

/// Opens outbound connections and returns the final wire stream.
#[async_trait]
pub trait Connector: Send + Sync + 'static {
    /// Stream produced by this connector.
    type Stream: ConnStream;

    /// Remote target for outbound connection.
    type Target;

    /// Connect to one remote target.
    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream>;
}
