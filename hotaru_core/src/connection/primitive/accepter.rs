//! Inbound primitive for upgrading an accepted raw stream.

use core::future::Future;

use crate::connection::ConnStream;

/// Converts an accepted raw stream into the final wire stream.
pub trait Accepter: Send + Sync + 'static {
    /// Raw stream accepted by the inbound runtime.
    type Raw: Send + 'static;

    /// Stream produced by this accepter.
    type Stream: ConnStream;

    /// Upgrade or pass through one accepted raw stream.
    fn upgrade(
        &self,
        raw: Self::Raw,
    ) -> impl Future<Output = std::io::Result<Self::Stream>> + Send;
}
