use core::future::Future;
use core::time::Duration;

use crate::marker::MaybeSend;

use super::{AsyncMutexCap, Either, OnceCellCap};

/// Universal async runtime capabilities.
/// Carries spawn, time, select, and sync hooks used by framework code.
pub trait RuntimeSpec: 'static {
    /// Join handle produced by [`spawn`](RuntimeSpec::spawn). Awaitable;
    /// yields `Result<T, JoinError>` on completion.
    type JoinHandle<T: MaybeSend + 'static>: Future<Output = Result<T, Self::JoinError>>
        + MaybeSend
        + 'static;

    /// Error returned when a spawned task ends abnormally.
    type JoinError: core::error::Error + MaybeSend + Sync + 'static;

    /// Detached spawn. The spawned future runs for side effects only.
    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + MaybeSend + 'static;

    /// Typed spawn. Returns a join handle for the future's output.
    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + MaybeSend + 'static,
        F::Output: MaybeSend + 'static;

    /// Runtime-specific instant type used by deadlines.
    type Instant: Copy + MaybeSend + Sync + 'static;

    /// Runtime-specific timeout error.
    type TimeoutError: core::error::Error + MaybeSend + Sync + 'static;

    /// Returns the runtime's current instant.
    fn now() -> Self::Instant;

    /// Adds a duration to a runtime instant.
    fn instant_plus(instant: Self::Instant, dur: Duration) -> Self::Instant;

    /// Sleeps for a relative duration.
    fn sleep(dur: Duration) -> impl Future<Output = ()> + MaybeSend + 'static;

    /// Sleeps until an absolute runtime deadline.
    fn sleep_until(deadline: Self::Instant) -> impl Future<Output = ()> + MaybeSend + 'static;

    /// Runs a future with a relative timeout.
    fn timeout<F>(
        dur: Duration,
        future: F,
    ) -> impl Future<Output = Result<F::Output, Self::TimeoutError>> + MaybeSend
    where
        F: Future + MaybeSend,
        F::Output: MaybeSend;

    /// Races two futures and returns whichever completes first.
    fn select2<A, B>(a: A, b: B) -> impl Future<Output = Either<A::Output, B::Output>> + MaybeSend
    where
        A: Future + MaybeSend,
        B: Future + MaybeSend,
        A::Output: MaybeSend,
        B::Output: MaybeSend;

    /// Default shutdown signal for [`Server::run`](crate::app::server::Server::run).
    ///
    /// The default is "never stop"; runtimes with a natural process-level
    /// signal may override it (Tokio uses Ctrl+C). Callers that need a custom
    /// stop source should use `Server::run_until(stop)`.
    fn default_stop() -> crate::marker::BoxFuture<'static, ()> {
        alloc::boxed::Box::pin(core::future::pending())
    }

    /// Runtime-specific one-time-init cell.
    type OnceCell<T: MaybeSend + Sync + 'static>: OnceCellCap<T>;

    /// Runtime-specific async mutex.
    type AsyncMutex<T: MaybeSend + 'static>: AsyncMutexCap<T>;
}
