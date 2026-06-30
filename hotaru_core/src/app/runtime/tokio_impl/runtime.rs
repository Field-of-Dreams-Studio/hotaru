use core::future::Future;
use core::time::Duration;

use crate::app::runtime::spec::{Either, RuntimeSpec};

use super::{TokioMutex, TokioOnceCell};

/// Tokio-backed runtime. Spawn forwards to `tokio::spawn`; time/select/sync
/// forward to the matching `tokio::*` primitives.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokioRuntime;

impl RuntimeSpec for TokioRuntime {
    type JoinHandle<T: Send + 'static> = tokio::task::JoinHandle<T>;
    type JoinError = tokio::task::JoinError;

    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // `tokio::spawn` returns a must_use JoinHandle; drop it explicitly
        // to signal fire-and-forget intent.
        let _ = tokio::spawn(future);
    }

    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(future)
    }

    type Instant = tokio::time::Instant;
    type TimeoutError = tokio::time::error::Elapsed;

    fn now() -> Self::Instant {
        tokio::time::Instant::now()
    }

    fn instant_plus(instant: Self::Instant, dur: Duration) -> Self::Instant {
        instant + dur
    }

    fn sleep(dur: Duration) -> impl Future<Output = ()> + Send + 'static {
        tokio::time::sleep(dur)
    }

    fn sleep_until(deadline: Self::Instant) -> impl Future<Output = ()> + Send + 'static {
        tokio::time::sleep_until(deadline)
    }

    async fn timeout<F>(dur: Duration, future: F) -> Result<F::Output, Self::TimeoutError>
    where
        F: Future + Send,
        F::Output: Send,
    {
        tokio::time::timeout(dur, future).await
    }

    async fn select2<A, B>(a: A, b: B) -> Either<A::Output, B::Output>
    where
        A: Future + Send,
        B: Future + Send,
        A::Output: Send,
        B::Output: Send,
    {
        tokio::select! {
            r = a => Either::Left(r),
            r = b => Either::Right(r),
        }
    }

    type OnceCell<T: Send + Sync + 'static> = TokioOnceCell<T>;
    type AsyncMutex<T: Send + 'static> = TokioMutex<T>;
}
