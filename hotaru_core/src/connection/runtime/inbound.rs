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

    /// Bind and construct the inbound runtime.
    async fn bind(target: Self::BindTarget) -> std::io::Result<Self>
    where
        Self: Sized;

    /// Wait for one inbound wire.
    async fn accept(&self) -> std::io::Result<Self::Wire>;
}
