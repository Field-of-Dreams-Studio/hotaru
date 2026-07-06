use alloc::{boxed::Box, sync::Arc};
use core::{
    cell::UnsafeCell,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use embassy_futures::select;
use embassy_sync::{channel::Channel, signal::Signal};
use hotaru_core::{
    app::runtime::{BoxFuture, Either},
    marker::MaybeSend,
};

use crate::mutex::EmbassyRawMutex;

/// A boxed Hotaru job ready to be driven by an Embassy worker.
pub type EmbassyJob = BoxFuture<'static, ()>;

/// Global job queue storage for one configured Embassy runtime.
///
/// The capacity is a const generic so the macro-generated runtime can keep the
/// queue static without baking a fixed size into this backend.
pub struct EmbassyJobQueue<const N: usize>(Channel<EmbassyRawMutex, EmbassyJob, N>);

// SAFETY: the queue is used as a global scheduler handoff. Under `spawn_send`
// the raw mutex is critical-section based; under `spawn_local` callers must use
// the single Embassy executor contract already required by this backend.
unsafe impl<const N: usize> Sync for EmbassyJobQueue<N> {}

impl<const N: usize> EmbassyJobQueue<N> {
    /// Create an empty job queue.
    pub const fn new() -> Self {
        Self(Channel::new())
    }

    fn try_send(&'static self, job: EmbassyJob) -> Result<(), EmbassyJoinError> {
        self.0
            .try_send(job)
            .map_err(|_| EmbassyJoinError::QueueFull)
    }

    async fn receive(&'static self) -> EmbassyJob {
        self.0.receive().await
    }
}

/// Initialization flag for one configured Embassy runtime.
pub struct EmbassyRuntimeState(UnsafeCell<bool>);

// SAFETY: all mutation is protected by `critical_section::with`.
unsafe impl Sync for EmbassyRuntimeState {}

impl EmbassyRuntimeState {
    /// Create an uninitialized runtime state.
    pub const fn new() -> Self {
        Self(UnsafeCell::new(false))
    }

    /// Mark the runtime as initialized, returning true only for the first call.
    pub fn initialize_once(&'static self) -> bool {
        critical_section::with(|_| unsafe {
            let initialized = &mut *self.0.get();
            let start_workers = !*initialized;
            *initialized = true;
            start_workers
        })
    }

    /// Returns whether the runtime has already been initialized.
    pub fn is_initialized(&'static self) -> bool {
        critical_section::with(|_| unsafe { *self.0.get() })
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

enum JoinState<T: MaybeSend + 'static> {
    Waiting(Arc<Signal<EmbassyRawMutex, T>>),
    Ready(Option<Result<T, EmbassyJoinError>>),
    Done,
}

/// Awaitable handle returned by a configured Embassy runtime.
pub struct EmbassyJoinHandle<T: MaybeSend + 'static> {
    state: JoinState<T>,
}

impl<T: MaybeSend + 'static> EmbassyJoinHandle<T> {
    fn waiting(signal: Arc<Signal<EmbassyRawMutex, T>>) -> Self {
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

impl<T: MaybeSend + 'static> Unpin for EmbassyJoinHandle<T> {}

impl<T: MaybeSend + 'static> Future for EmbassyJoinHandle<T> {
    type Output = Result<T, EmbassyJoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match core::mem::replace(&mut this.state, JoinState::Done) {
            JoinState::Waiting(signal) => {
                let poll = {
                    let mut wait = signal.wait();
                    unsafe { Pin::new_unchecked(&mut wait) }.poll(cx)
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
pub async fn run_queued_jobs<const N: usize>(queue: &'static EmbassyJobQueue<N>) {
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
    let signal = Arc::new(Signal::<EmbassyRawMutex, F::Output>::new());
    let task_signal = Arc::clone(&signal);
    let task = async move {
        let output = future.await;
        task_signal.signal(output);
    };

    match spawn_task(state, queue, task) {
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
/// );
/// ```
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
        );
    };
    ($worker_count:expr, $job_queue_capacity:expr $(,)?) => {
        $crate::define_runtime_worker_pool!(
            pub EmbassyRuntime,
            worker_count = $worker_count,
            job_queue_capacity = $job_queue_capacity,
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
        );
    };
    (
        $vis:vis $runtime:ident,
        worker_count = $worker_count:expr,
        job_queue_capacity = $job_queue_capacity:expr $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, Default)]
        $vis struct $runtime;

        mod __hotaru_rt_embassy_runtime {
            use super::$runtime;

            const WORKER_COUNT: usize = $worker_count;
            const JOB_QUEUE_CAPACITY: usize = $job_queue_capacity;

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
            static JOB_QUEUE: $crate::__private::EmbassyJobQueue<JOB_QUEUE_CAPACITY> =
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
                > = $crate::EmbassyJoinHandle<T>;
                type JoinError = $crate::EmbassyJoinError;

                fn spawn_detached<F>(future: F)
                where
                    F: ::core::future::Future<Output = ()>
                        + $crate::__private::hotaru_core::marker::MaybeSend
                        + 'static,
                {
                    $crate::__private::spawn_task(&RUNTIME_STATE, &JOB_QUEUE, future)
                        .expect("failed to spawn detached embassy task");
                }

                fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
                where
                    F: ::core::future::Future
                        + $crate::__private::hotaru_core::marker::MaybeSend
                        + 'static,
                    F::Output: $crate::__private::hotaru_core::marker::MaybeSend + 'static,
                {
                    $crate::__private::spawn_join(&RUNTIME_STATE, &JOB_QUEUE, future)
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
                    T: $crate::__private::hotaru_core::marker::MaybeSend
                        + Sync
                        + 'static,
                > = $crate::__private::EmbassyOnceCell<T>;
                type AsyncMutex<
                    T: $crate::__private::hotaru_core::marker::MaybeSend + 'static,
                > = $crate::__private::EmbassyMutex<T>;
            }
        }
    };
}
