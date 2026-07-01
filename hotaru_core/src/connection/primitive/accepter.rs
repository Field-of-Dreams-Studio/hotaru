//! Inbound primitive for upgrading an accepted raw stream.

use core::future::Future;

use crate::connection::{ConnStream, MaybeSend};

/// Converts an accepted raw stream into the final wire stream.
///
/// `Error` is an associated type so embedded transports (whose backends
/// error with something other than `std::io::Error`) can implement this
/// trait. Std-flavoured impls typically set `type Error = std::io::Error;`.
pub trait Accepter: Send + Sync + 'static {
    /// Raw stream accepted by the inbound runtime.
    type Raw: Send + 'static;

    /// Stream produced by this accepter.
    type Stream: ConnStream;

    /// Error returned by `upgrade`. Std-flavoured impls typically pick
    /// `std::io::Error`; embedded impls pick their transport's own type.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Upgrade or pass through one accepted raw stream.
    fn upgrade(
        &self,
        raw: Self::Raw,
    ) -> impl Future<Output = Result<Self::Stream, Self::Error>> + MaybeSend;
}
