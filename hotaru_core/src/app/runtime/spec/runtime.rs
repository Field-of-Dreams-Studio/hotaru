use core::future::Future;
use core::time::Duration;

use crate::marker::MaybeSend;

use super::{AsyncMutexCap, Either, OnceCellCap};

/// Universal async runtime capabilities: spawn, time, select, and sync.
///
/// Framework code (`Server`, `Client`, HTTP channel loops, …) reaches
/// every runtime primitive through this trait — never through direct
/// `tokio::*` / `embassy::*` calls. Implementors bridge one concrete
/// executor onto this surface.
///
/// # Bounds — `MaybeSend`
///
/// Every future bound is `+ MaybeSend`, not `+ Send`. Under
/// `feature = "spawn_send"` (default) `MaybeSend` **is** `Send`, so
/// multi-threaded runtimes get the usual `Send`-required shape. Under
/// `feature = "spawn_local"` it is a vacuous marker, so single-executor
/// runtimes (embassy, `LocalSet`) can supply `!Send` futures. Associated
/// types stay unconditionally `Sync + 'static` because the framework
/// shares them via `Arc`.
///
/// # Cancellation contract
///
/// Every method must be **cancel-safe**: dropping the returned future
/// mid-poll must not leak resources, corrupt runtime state, or orphan a
/// spawned task. The framework relies on this for its `select2` and
/// `timeout` racing patterns.
pub trait RuntimeSpec: 'static {
    /// Awaitable handle produced by [`spawn`](RuntimeSpec::spawn).
    ///
    /// Yields `Ok(output)` on normal completion and `Err(Self::JoinError)`
    /// on panic, external cancellation, or executor shutdown. Whether
    /// dropping the handle detaches or aborts the task is backend-defined.
    type JoinHandle<T: MaybeSend + 'static>: Future<Output = Result<T, Self::JoinError>>
        + MaybeSend
        + 'static;

    /// Error surfaced by [`JoinHandle`](RuntimeSpec::JoinHandle) on
    /// abnormal end. Concrete backends wrap panic payloads and
    /// cancellation reasons; framework code treats it as opaque.
    type JoinError: core::error::Error + MaybeSend + Sync + 'static;

    /// Fire-and-forget spawn. Suitable for per-connection accept loops
    /// and background timers whose completion the framework does not
    /// observe. Must schedule the task before returning.
    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + MaybeSend + 'static;

    /// Typed spawn. Returns a join handle whose output matches `F::Output`.
    /// Used by `Client::call_fn` / `Client::call_url`. Backends without a
    /// native generic-over-`T` spawn (embassy's `SpawnToken<S>` model)
    /// must synthesise a shim — typically boxing the future through a
    /// fixed-size trampoline pool.
    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + MaybeSend + 'static,
        F::Output: MaybeSend + 'static;

    /// Runtime-specific monotonic instant. Wall-clock time is **not**
    /// acceptable — it can jump backward under NTP adjustment.
    type Instant: Copy + MaybeSend + Sync + 'static;

    /// Error yielded by [`timeout`](RuntimeSpec::timeout) when the
    /// deadline fires before the inner future completes.
    type TimeoutError: core::error::Error + MaybeSend + Sync + 'static;

    /// Current runtime instant. Cheap; must not perform blocking syscalls.
    fn now() -> Self::Instant;

    /// Advance `instant` by `dur`. Must **saturate** rather than wrap on
    /// overflow so callers can hand it user-derived deadlines safely.
    fn instant_plus(instant: Self::Instant, dur: Duration) -> Self::Instant;

    /// Sleep for a relative duration. Cancel-safe: dropping the future
    /// releases the timer slot.
    fn sleep(dur: Duration) -> impl Future<Output = ()> + MaybeSend + 'static;

    /// Sleep until an absolute deadline. If `deadline` is already past,
    /// the future may complete on the first poll.
    fn sleep_until(deadline: Self::Instant) -> impl Future<Output = ()> + MaybeSend + 'static;

    /// Race `future` against a relative deadline. On timeout, `future`
    /// **is dropped** — the runtime does not retain it. Bodies that
    /// aren't cancel-safe will observe a torn state.
    fn timeout<F>(
        dur: Duration,
        future: F,
    ) -> impl Future<Output = Result<F::Output, Self::TimeoutError>> + MaybeSend
    where
        F: Future + MaybeSend,
        F::Output: MaybeSend;

    /// Race two futures. The loser is **dropped** as soon as the winner
    /// resolves. No fairness guarantee — each backend may bias
    /// arbitrarily.
    fn select2<A, B>(a: A, b: B) -> impl Future<Output = Either<A::Output, B::Output>> + MaybeSend
    where
        A: Future + MaybeSend,
        B: Future + MaybeSend,
        A::Output: MaybeSend,
        B::Output: MaybeSend;

    /// Default shutdown signal for [`Server::run`](crate::app::server::Server::run).
    /// Defaults to `pending()` — "never stop". Runtimes with a
    /// process-level signal should override (Tokio uses Ctrl+C); callers
    /// wanting a custom source should use `Server::run_until(stop)`.
    fn default_stop() -> crate::marker::BoxFuture<'static, ()> {
        alloc::boxed::Box::pin(core::future::pending())
    }

    /// Backend's async one-time-init cell. Framework fields like
    /// `Server::inbound` are typed `Rt::OnceCell<Arc<...>>` and
    /// materialised on first use.
    type OnceCell<T: MaybeSend + Sync + 'static>: OnceCellCap<T>;

    /// Backend's async mutex — the "held across `.await`" flavour, not
    /// the sync [`PMutex`](crate::marker::PMutex) used for short critical
    /// sections.
    type AsyncMutex<T: MaybeSend + 'static>: AsyncMutexCap<T>;
}
