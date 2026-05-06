//! Client-side runtime that opens outbound wire streams.

use async_trait::async_trait;

use crate::connection::ConnStream;

/// Outbound runtime that opens final wire streams.
#[async_trait]
pub trait Outbound: Send + Sync + 'static {
    /// Wire stream produced by this outbound runtime.
    type Wire: ConnStream;

    /// Remote target plus any transport-specific connection config.
    type ConnectTarget: Clone + Send + Sync + 'static;

    /// Connect to one outbound target.
    async fn connect(target: Self::ConnectTarget) -> std::io::Result<Self::Wire>;
}
