//! Server-side runtime that accepts inbound wire streams.

use core::future::Future;

use crate::connection::{ConnStream, MaybeSend};

/// Bound inbound runtime that accepts final wire streams.
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
    fn bind(
        target: Self::BindTarget,
    ) -> impl Future<Output = Result<Self, Self::Error>> + MaybeSend
    where
        Self: Sized;

    /// Wait for one inbound wire.
    fn accept(&self) -> impl Future<Output = Result<Self::Wire, Self::Error>> + MaybeSend;
}
