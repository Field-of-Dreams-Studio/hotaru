use core::{
    cell::Cell,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use embassy_futures::select;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::{channel::Channel, signal::Signal};
use hotaru_core::{app::runtime::Either, prelude::*};

use crate::mutex::EmbassyRawMutex;

/// A boxed Hotaru job ready to be driven by an Embassy worker.
pub type EmbassyJob = BoxFuture<'static, ()>;

/// Global job queue storage for one configured Embassy runtime.
///
/// The capacity is a const generic so the macro-generated runtime can keep the
/// queue static without baking a fixed size into this backend.
pub struct EmbassyJobQueue<const N: usize, M = EmbassyRawMutex>
where
    M: RawMutex,
{
    inner: Channel<M, EmbassyJob, N>,
}

// SAFETY: the queue is used as a global scheduler handoff. Under `spawn_send`
// `Channel` is naturally `Sync` when the selected raw mutex is `Sync`; under
// `spawn_local`, callers must use the single Embassy executor contract already
// required by this backend.
#[cfg(feature = "spawn_local")]
unsafe impl<const N: usize, M> Sync for EmbassyJobQueue<N, M> where M: RawMutex {}

impl<const N: usize, M> EmbassyJobQueue<N, M>
where
    M: RawMutex,
{
    /// Create an empty job queue.
    pub const fn new() -> Self {
        Self {
            inner: Channel::new(),
        }
    }

    fn try_send(&'static self, job: EmbassyJob) -> Result<(), EmbassyJoinError> {
        self.inner
            .try_send(job)
            .map_err(|_| EmbassyJoinError::QueueFull)
    }

    async fn receive(&'static self) -> EmbassyJob {
        self.inner.receive().await
    }
}

impl<const N: usize, M> Default for EmbassyJobQueue<N, M>
where
    M: RawMutex,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Initialization flag for one configured Embassy runtime.
pub struct EmbassyRuntimeState(critical_section::Mutex<Cell<bool>>);

impl EmbassyRuntimeState {
    /// Create an uninitialized runtime state.
    pub const fn new() -> Self {
        Self(critical_section::Mutex::new(Cell::new(false)))
    }

    /// Mark the runtime as initialized, returning true only for the first call.
    pub fn initialize_once(&'static self) -> bool {
        critical_section::with(|cs| {
            let initialized = self.0.borrow(cs);
            let start_workers = !initialized.get();
            initialized.set(true);
            start_workers
        })
    }

    /// Returns whether the runtime has already been initialized.
    pub fn is_initialized(&'static self) -> bool {
        critical_section::with(|cs| self.0.borrow(cs).get())
    }
}

impl Default for EmbassyRuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned by Embassy join handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbassyJoinError {
    /// The configured Embassy runtime has not installed an Embassy spawner yet.
    SpawnerNotInitialized,
    /// The bounded runtime job queue is full.
    QueueFull,
    /// The join handle was polled again after completion.
    PolledAfterCompletion,
}

impl fmt::Display for EmbassyJoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpawnerNotInitialized => f.write_str("embassy spawner is not initialized"),
            Self::QueueFull => f.write_str("embassy runtime job queue is full"),
            Self::PolledAfterCompletion => {
                f.write_str("embassy join handle polled after completion")
            }
        }
    }
}

impl core::error::Error for EmbassyJoinError {}

/// Error returned when an Embassy timeout expires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbassyTimeoutError;

impl fmt::Display for EmbassyTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("embassy timeout elapsed")
    }
}

impl core::error::Error for EmbassyTimeoutError {}

enum JoinState<T, M>
where
    T: MaybeSend + 'static,
    M: RawMutex,
{
    Waiting(Arc<Signal<M, T>>),
    Ready(Option<Result<T, EmbassyJoinError>>),
    Done,
}

/// Awaitable handle returned by a configured Embassy runtime.
pub struct EmbassyJoinHandle<T, M = EmbassyRawMutex>
where
    T: MaybeSend + 'static,
    M: RawMutex,
{
    state: JoinState<T, M>,
}

impl<T, M> EmbassyJoinHandle<T, M>
where
    T: MaybeSend + 'static,
    M: RawMutex,
{
    fn waiting(signal: Arc<Signal<M, T>>) -> Self {
        Self {
            state: JoinState::Waiting(signal),
        }
    }

    fn failed(error: EmbassyJoinError) -> Self {
        Self {
            state: JoinState::Ready(Some(Err(error))),
        }
    }
}

impl<T, M> Unpin for EmbassyJoinHandle<T, M>
where
    T: MaybeSend + 'static,
    M: RawMutex,
{
}

impl<T, M> Future for EmbassyJoinHandle<T, M>
where
    T: MaybeSend + 'static,
    M: RawMutex,
{
    type Output = Result<T, EmbassyJoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match core::mem::replace(&mut this.state, JoinState::Done) {
            JoinState::Waiting(signal) => {
                let poll = {
                    let mut wait = core::pin::pin!(signal.wait());
                    wait.as_mut().poll(cx)
                };

                match poll {
                    Poll::Ready(output) => Poll::Ready(Ok(output)),
                    Poll::Pending => {
                        this.state = JoinState::Waiting(signal);
                        Poll::Pending
                    }
                }
            }
            JoinState::Ready(mut result) => Poll::Ready(
                result
                    .take()
                    .unwrap_or(Err(EmbassyJoinError::PolledAfterCompletion)),
            ),
            JoinState::Done => Poll::Pending,
        }
    }
}

pub fn to_embassy_duration(duration: Duration) -> embassy_time::Duration {
    u64::try_from(duration.as_nanos())
        .ok()
        .and_then(embassy_time::Duration::try_from_nanos)
        .unwrap_or(embassy_time::Duration::MAX)
}

/// Run one configured runtime worker forever.
pub async fn run_queued_jobs<const N: usize, M>(queue: &'static EmbassyJobQueue<N, M>)
where
    M: RawMutex + 'static,
{
    loop {
        let job = queue.receive().await;
        job.await;
    }
}

/// Queue a detached task onto one configured runtime.
pub fn spawn_task<F, const N: usize>(
    state: &'static EmbassyRuntimeState,
    queue: &'static EmbassyJobQueue<N>,
    future: F,
) -> Result<(), EmbassyJoinError>
where
    F: Future<Output = ()> + MaybeSend + 'static,
{
    spawn_task_with_mutex(state, queue, future)
}

/// Queue a detached task onto one configured runtime with a custom raw mutex.
pub fn spawn_task_with_mutex<F, const N: usize, M>(
    state: &'static EmbassyRuntimeState,
    queue: &'static EmbassyJobQueue<N, M>,
    future: F,
) -> Result<(), EmbassyJoinError>
where
    F: Future<Output = ()> + MaybeSend + 'static,
    M: RawMutex + 'static,
{
    if !state.is_initialized() {
        return Err(EmbassyJoinError::SpawnerNotInitialized);
    }

    queue.try_send(Box::pin(future))
}

/// Queue a task and return a join handle for its output.
pub fn spawn_join<F, const N: usize>(
    state: &'static EmbassyRuntimeState,
    queue: &'static EmbassyJobQueue<N>,
    future: F,
) -> EmbassyJoinHandle<F::Output>
where
    F: Future + MaybeSend + 'static,
    F::Output: MaybeSend + 'static,
{
    spawn_join_with_mutex(state, queue, future)
}

/// Queue a task onto one configured runtime with a custom raw mutex.
#[cfg(feature = "spawn_send")]
pub fn spawn_join_with_mutex<F, const N: usize, M>(
    state: &'static EmbassyRuntimeState,
    queue: &'static EmbassyJobQueue<N, M>,
    future: F,
) -> EmbassyJoinHandle<F::Output, M>
where
    F: Future + MaybeSend + 'static,
    F::Output: MaybeSend + 'static,
    M: RawMutex + Send + Sync + 'static,
{
    let signal = Arc::new(Signal::<M, F::Output>::new());
    let task_signal = Arc::clone(&signal);
    let task = async move {
        let output = future.await;
        task_signal.signal(output);
    };

    match spawn_task_with_mutex(state, queue, task) {
        Ok(()) => EmbassyJoinHandle::waiting(signal),
        Err(error) => EmbassyJoinHandle::failed(error),
    }
}

/// Queue a task onto one configured runtime with a custom raw mutex.
#[cfg(feature = "spawn_local")]
pub fn spawn_join_with_mutex<F, const N: usize, M>(
    state: &'static EmbassyRuntimeState,
    queue: &'static EmbassyJobQueue<N, M>,
    future: F,
) -> EmbassyJoinHandle<F::Output, M>
where
    F: Future + MaybeSend + 'static,
    F::Output: MaybeSend + 'static,
    M: RawMutex + 'static,
{
    let signal = Arc::new(Signal::<M, F::Output>::new());
    let task_signal = Arc::clone(&signal);
    let task = async move {
        let output = future.await;
        task_signal.signal(output);
    };

    match spawn_task_with_mutex(state, queue, task) {
        Ok(()) => EmbassyJoinHandle::waiting(signal),
        Err(error) => EmbassyJoinHandle::failed(error),
    }
}

/// Race two futures using Embassy's select primitive.
pub async fn select2<A, B>(a: A, b: B) -> Either<A::Output, B::Output>
where
    A: Future + MaybeSend,
    B: Future + MaybeSend,
    A::Output: MaybeSend,
    B::Output: MaybeSend,
{
    match select::select(a, b).await {
        select::Either::First(output) => Either::Left(output),
        select::Either::Second(output) => Either::Right(output),
    }
}

/// Define a configured Embassy-backed Hotaru runtime.
///
/// Hotaru's runtime trait schedules through static methods, so the Embassy job
/// queue is intentionally global for each generated runtime type. All fixed
/// capacities are supplied by the macro caller:
///
/// ```ignore
/// hotaru_rt_embassy::define_runtime_worker_pool!(
///     pub AppRuntime,
///     worker_count = 4,
///     job_queue_capacity = 32,
///     raw_mutex = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
/// );
/// ```
///
/// Fixed points exposed by this macro:
///
/// - `worker_count`: configurable worker pool size.
/// - `job_queue_capacity`: configurable bounded queue capacity.
/// - `raw_mutex`: configurable Embassy raw mutex used by the queue, join
///   signals, once-cell init lock, and async mutex.
///
/// The shorthand `define_runtime_worker_pool!(N)` creates `pub EmbassyRuntime`
/// with both worker count and queue capacity set to `N`.
#[macro_export]
macro_rules! define_runtime_worker_pool {
    ($worker_count:expr $(,)?) => {
        $crate::define_runtime_worker_pool!(
            pub EmbassyRuntime,
            worker_count = $worker_count,
            job_queue_capacity = $worker_count,
            raw_mutex = $crate::__private::EmbassyRawMutex,
        );
    };
    ($worker_count:expr, $job_queue_capacity:expr $(,)?) => {
        $crate::define_runtime_worker_pool!(
            pub EmbassyRuntime,
            worker_count = $worker_count,
            job_queue_capacity = $job_queue_capacity,
            raw_mutex = $crate::__private::EmbassyRawMutex,
        );
    };
    (
        $vis:vis $runtime:ident,
        workers = $worker_count:expr,
        queue = $job_queue_capacity:expr $(,)?
    ) => {
        $crate::define_runtime_worker_pool!(
            $vis $runtime,
            worker_count = $worker_count,
            job_queue_capacity = $job_queue_capacity,
            raw_mutex = $crate::__private::EmbassyRawMutex,
        );
    };
    (
        $vis:vis $runtime:ident,
        workers = $worker_count:expr,
        queue = $job_queue_capacity:expr,
        raw_mutex = $raw_mutex:ty $(,)?
    ) => {
        $crate::define_runtime_worker_pool!(
            $vis $runtime,
            worker_count = $worker_count,
            job_queue_capacity = $job_queue_capacity,
            raw_mutex = $raw_mutex,
        );
    };
    (
        $vis:vis $runtime:ident,
        worker_count = $worker_count:expr,
        job_queue_capacity = $job_queue_capacity:expr $(,)?
    ) => {
        $crate::define_runtime_worker_pool!(
            $vis $runtime,
            worker_count = $worker_count,
            job_queue_capacity = $job_queue_capacity,
            raw_mutex = $crate::__private::EmbassyRawMutex,
        );
    };
    (
        $vis:vis $runtime:ident,
        worker_count = $worker_count:expr,
        job_queue_capacity = $job_queue_capacity:expr,
        raw_mutex = $raw_mutex:ty $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, Default)]
        $vis struct $runtime;

        const _: () = {
            const WORKER_COUNT: usize = $worker_count;
            const JOB_QUEUE_CAPACITY: usize = $job_queue_capacity;
            type RawMutex = $raw_mutex;

            const _: () = {
                assert!(
                    WORKER_COUNT > 0,
                    "hotaru_rt_embassy: worker_count must be greater than 0",
                );
                assert!(
                    JOB_QUEUE_CAPACITY > 0,
                    "hotaru_rt_embassy: job_queue_capacity must be greater than 0",
                );
            };

            static RUNTIME_STATE: $crate::__private::EmbassyRuntimeState =
                $crate::__private::EmbassyRuntimeState::new();
            static JOB_QUEUE: $crate::__private::EmbassyJobQueue<JOB_QUEUE_CAPACITY, RawMutex> =
                $crate::__private::EmbassyJobQueue::new();

            #[$crate::__private::embassy_executor::task(pool_size = WORKER_COUNT)]
            async fn hotaru_job_worker() {
                $crate::__private::run_queued_jobs(&JOB_QUEUE).await;
            }

            impl $runtime {
                /// Number of Embassy workers in this generated runtime.
                pub const WORKER_COUNT: usize = WORKER_COUNT;
                /// Capacity of this generated runtime's global job queue.
                pub const JOB_QUEUE_CAPACITY: usize = JOB_QUEUE_CAPACITY;

                /// Installs the Embassy spawner and starts this runtime's workers.
                ///
                /// Call this once from the Embassy entry task before using Hotaru APIs
                /// that spawn tasks.
                pub fn init(spawner: $crate::__private::embassy_executor::Spawner) {
                    if RUNTIME_STATE.initialize_once() {
                        for _ in 0..WORKER_COUNT {
                            spawner.spawn(
                                hotaru_job_worker()
                                    .expect("failed to spawn hotaru embassy job worker"),
                            );
                        }
                    }
                }

                /// Returns whether this generated runtime has been initialized.
                pub fn is_initialized() -> bool {
                    RUNTIME_STATE.is_initialized()
                }
            }

            impl $crate::__private::hotaru_core::app::runtime::RuntimeSpec for $runtime {
                type JoinHandle<
                    T: $crate::__private::hotaru_core::marker::MaybeSend + 'static,
                > = $crate::EmbassyJoinHandle<T, RawMutex>;
                type JoinError = $crate::EmbassyJoinError;

                fn spawn_detached<F>(future: F)
                where
                    F: ::core::future::Future<Output = ()>
                        + $crate::__private::hotaru_core::marker::MaybeSend
                        + 'static,
                {
                    $crate::__private::spawn_task_with_mutex(&RUNTIME_STATE, &JOB_QUEUE, future)
                        .expect("failed to spawn detached embassy task");
                }

                fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
                where
                    F: ::core::future::Future
                        + $crate::__private::hotaru_core::marker::MaybeSend
                        + 'static,
                    F::Output: $crate::__private::hotaru_core::marker::MaybeSend + 'static,
                {
                    $crate::__private::spawn_join_with_mutex(&RUNTIME_STATE, &JOB_QUEUE, future)
                }

                type Instant = $crate::__private::embassy_time::Instant;
                type TimeoutError = $crate::EmbassyTimeoutError;

                fn now() -> Self::Instant {
                    $crate::__private::embassy_time::Instant::now()
                }

                fn instant_plus(
                    instant: Self::Instant,
                    dur: ::core::time::Duration,
                ) -> Self::Instant {
                    instant.saturating_add($crate::__private::to_embassy_duration(dur))
                }

                fn sleep(
                    dur: ::core::time::Duration,
                ) -> impl ::core::future::Future<Output = ()>
                       + $crate::__private::hotaru_core::marker::MaybeSend
                       + 'static {
                    $crate::__private::embassy_time::Timer::after(
                        $crate::__private::to_embassy_duration(dur),
                    )
                }

                fn sleep_until(
                    deadline: Self::Instant,
                ) -> impl ::core::future::Future<Output = ()>
                       + $crate::__private::hotaru_core::marker::MaybeSend
                       + 'static {
                    $crate::__private::embassy_time::Timer::at(deadline)
                }

                async fn timeout<F>(
                    dur: ::core::time::Duration,
                    future: F,
                ) -> Result<F::Output, Self::TimeoutError>
                where
                    F: ::core::future::Future
                        + $crate::__private::hotaru_core::marker::MaybeSend,
                    F::Output: $crate::__private::hotaru_core::marker::MaybeSend,
                {
                    $crate::__private::embassy_time::with_timeout(
                        $crate::__private::to_embassy_duration(dur),
                        future,
                    )
                    .await
                    .map_err(|_| $crate::EmbassyTimeoutError)
                }

                async fn select2<A, B>(
                    a: A,
                    b: B,
                ) -> $crate::__private::hotaru_core::app::runtime::Either<A::Output, B::Output>
                where
                    A: ::core::future::Future
                        + $crate::__private::hotaru_core::marker::MaybeSend,
                    B: ::core::future::Future
                        + $crate::__private::hotaru_core::marker::MaybeSend,
                    A::Output: $crate::__private::hotaru_core::marker::MaybeSend,
                    B::Output: $crate::__private::hotaru_core::marker::MaybeSend,
                {
                    $crate::__private::select2(a, b).await
                }

                type OnceCell<
                    T: $crate::__private::hotaru_core::marker::MaybeSendSync + 'static,
                > = $crate::__private::EmbassyOnceCell<T, RawMutex>;
                type AsyncMutex<
                    T: $crate::__private::hotaru_core::marker::MaybeSend + 'static,
                > = $crate::__private::EmbassyMutex<T, RawMutex>;
            }
        };
    };
}
