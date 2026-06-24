//! Server-side runtime that accepts inbound wire streams.

use async_trait::async_trait;

use crate::connection::ConnStream;

/// Bound inbound runtime that accepts final wire streams.
#[async_trait]
pub trait Inbound: Send + Sync + 'static {
    /// Wire stream produced by this runtime.
    type Wire: ConnStream;

    /// Local bind target and any transport-specific construction config.
    type BindTarget: Clone + Send + Sync + 'static;

    /// Error type returned by `bind` and `accept`. For std-flavoured
    /// transports this is typically `std::io::Error`; embedded transports
    /// pick their own. `TransportSpec` pins this to its `IoError`.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Bind and construct the inbound runtime.
    async fn bind(target: Self::BindTarget) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Wait for one inbound wire.
    async fn accept(&self) -> Result<Self::Wire, Self::Error>;
}
