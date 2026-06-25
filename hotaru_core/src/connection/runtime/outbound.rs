//! Client-side runtime that opens outbound wire streams.
//!
//! `Outbound` is **instance-based**, mirroring [`Inbound`]. The target is
//! baked into the instance at [`build`](Outbound::build) time; subsequent
//! [`connect`](Outbound::connect) calls are zero-arg and may return a
//! fresh wire, a pooled one, or a clone of a multiplexed connection —
//! depending on the concrete transport.
//!
//! [`Inbound`]: crate::connection::Inbound

use core::future::Future;

use crate::connection::ConnStream;

/// Outbound runtime that opens final wire streams.
pub trait Outbound: Send + Sync + 'static {
    /// Wire stream produced by this outbound runtime.
    type Wire: ConnStream;

    /// Remote target plus any transport-specific connection config.
    ///
    /// Packs whatever the transport needs (address, TLS config, pool size,
    /// keep-alive policy, …) into one type. Mirrors the convention of
    /// [`Inbound::BindTarget`](crate::connection::Inbound::BindTarget).
    type ConnectTarget: Clone + Send + Sync + 'static;

    /// Error type returned by `build` and `connect`. `TransportSpec`
    /// pins this to its `IoError` so server and client error surfaces
    /// share a single type per transport.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Build the outbound runtime bound to a target.
    ///
    /// The target lives on `Self` from this point on; the transport may
    /// pre-establish pools, prepare TLS state, resolve DNS once, etc.
    fn build(
        target: Self::ConnectTarget,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send
    where
        Self: Sized;

    /// Acquire one wire to the bound target.
    ///
    /// Implementations may return a freshly opened wire, a connection
    /// from a pool, a logical stream over a multiplexed connection, or
    /// anything else the transport considers a valid "one wire to the
    /// configured target."
    fn connect(&self) -> impl Future<Output = Result<Self::Wire, Self::Error>> + Send;
}
