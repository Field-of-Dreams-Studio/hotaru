//! Inbound primitive for upgrading an accepted raw stream.

use async_trait::async_trait;

use crate::connection::ConnStream;

/// Converts an accepted raw stream into the final wire stream.
#[async_trait]
pub trait Accepter: Send + Sync + 'static {
    /// Raw stream accepted by the inbound runtime.
    type Raw: Send + 'static;

    /// Stream produced by this accepter.
    type Stream: ConnStream;

    /// Upgrade or pass through one accepted raw stream.
    async fn upgrade(&self, raw: Self::Raw) -> std::io::Result<Self::Stream>;
}
