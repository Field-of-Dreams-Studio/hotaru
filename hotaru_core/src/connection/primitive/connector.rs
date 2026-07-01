//! Outbound primitive for opening final wire streams.

use core::future::Future;

use crate::connection::{ConnStream, MaybeSend};

/// Opens outbound connections and returns the final wire stream.
///
/// `Error` is an associated type so embedded transports (whose backends
/// error with something other than `std::io::Error`) can implement this
/// trait. Std-flavoured impls typically set `type Error = std::io::Error;`.
pub trait Connector: Send + Sync + 'static {
    /// Stream produced by this connector.
    type Stream: ConnStream;

    /// Remote target for outbound connection.
    type Target;

    /// Error returned by `connect`. Std-flavoured impls typically pick
    /// `std::io::Error`; embedded impls pick their transport's own type.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Connect to one remote target.
    fn connect(
        &self,
        target: Self::Target,
    ) -> impl Future<Output = Result<Self::Stream, Self::Error>> + MaybeSend;
}
