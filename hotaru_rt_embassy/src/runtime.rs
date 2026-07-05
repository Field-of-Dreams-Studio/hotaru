use alloc::{boxed::Box, sync::Arc};
use core::{
    cell::UnsafeCell,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use embassy_executor::Spawner;
use embassy_futures::select;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use hotaru_core::{
    app::runtime::{BoxFuture, Either, RuntimeSpec},
    marker::MaybeSend,
};

use crate::{EmbassyMutex, mutex::EmbassyRawMutex, once_cell::EmbassyOnceCell};
//引入工具
const WORKER_COUNT: usize = 4; //4个worker
const JOB_QUEUE_CAPACITY: usize = 16; //队列最多放16个任务

type Job = BoxFuture<'static, ()>;

struct JobQueue(Channel<EmbassyRawMutex, Job, JOB_QUEUE_CAPACITY>);

unsafe impl Sync for JobQueue {} //编译器不能自动确认它在多处共享是否安全

static JOB_QUEUE: JobQueue = JobQueue(Channel::new());

#[embassy_executor::task(pool_size = 4)]
async fn hotaru_job_worker() {
    loop {
        let job = JOB_QUEUE.0.receive().await;
        job.await;
    }
}
//一直循环，从一个队列中（4个任务）取出一个任务并执行，执行完继续下一个任务
/// Embassy-backed runtime.
///
/// Call [`EmbassyRuntime::init`] once from your Embassy entry task before using
/// Hotaru APIs that spawn tasks.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmbassyRuntime;
//这是一个代表 Embassy 运行时的空结构体。
//它自己不保存数据，只是一个“标识”：告诉 Hotaru 当前使用 Embassy 作为运行时后端。
struct RuntimeState(UnsafeCell<bool>);

unsafe impl Sync for RuntimeState {}

static INITIALIZED: RuntimeState = RuntimeState(UnsafeCell::new(false));
//这里记录 Embassy 运行时有没有初始化。
impl EmbassyRuntime {
    /// Installs the Embassy spawner used by [`RuntimeSpec::spawn`] and
    /// [`RuntimeSpec::spawn_detached`].
    ///
    /// Embassy exposes spawning through a `Spawner` value supplied by
    /// `#[embassy_executor::main]` or `Executor::run`. Hotaru's runtime trait is
    /// static, so this backend starts a small worker pool that accepts boxed
    /// Hotaru jobs through a bounded queue.
    pub fn init(spawner: Spawner) {
        let start_workers = critical_section::with(|_| unsafe {
            let initialized = &mut *INITIALIZED.0.get();
            let start_workers = !*initialized;
            *initialized = true;
            start_workers
        });

        if start_workers {
            for _ in 0..WORKER_COUNT {
                spawner
                    .spawn(hotaru_job_worker().expect("failed to spawn hotaru embassy job worker"));
            }
        }
    }

    fn is_initialized() -> bool {
        critical_section::with(|_| unsafe { *INITIALIZED.0.get() })
    }
}
//检查是否已经初始化过。
//如果没初始化过，就启动 4 个后台 worker。
/// Error returned by Embassy join handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbassyJoinError {
    /// [`EmbassyRuntime::init`] has not installed an Embassy spawner yet.
    SpawnerNotInitialized,
    /// The bounded runtime job queue is full.
    QueueFull,
    /// The join handle was polled again after completion.
    PolledAfterCompletion,
}
//这里定义等待任务结果时可能遇到的错误
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
//错误显示文本
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
//打印超时错误
enum JoinState<T: MaybeSend + 'static> {
    Waiting(Arc<Signal<EmbassyRawMutex, T>>),
    Ready(Option<Result<T, EmbassyJoinError>>),
    Done,
}
//这是任务结果句柄的内部状态
/// Awaitable handle returned by [`EmbassyRuntime::spawn`].
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
//waiting：任务已经成功排队，等待它完成。
//failed：任务没能成功排队，直接保存错误。
impl<T: MaybeSend + 'static> Unpin for EmbassyJoinHandle<T> {}
//这表示这个 handle 可以安全移动位置
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
//这是让 EmbassyJoinHandle 变成一个可以等待的东西。
//内部逻辑是：
//如果状态是 Waiting，就检查信号有没有来。
//如果任务完成了，返回 Ok(output)。
//如果还没完成，继续等待。
//如果状态是 Ready，直接返回之前保存的结果。
//如果已经 Done，就保持等待状态。
fn to_embassy_duration(duration: Duration) -> embassy_time::Duration {
    u64::try_from(duration.as_nanos())
        .ok()
        .and_then(embassy_time::Duration::try_from_nanos)
        .unwrap_or(embassy_time::Duration::MAX)
}
//这个函数负责把普通时间长度转换成 Embassy 能理解的时间长度。
//如果时间太大，超过 Embassy 能表示的范围，就用最大值，避免溢出。
fn spawn_task<F>(future: F) -> Result<(), EmbassyJoinError>
where
    F: Future<Output = ()> + MaybeSend + 'static,
{
    if !EmbassyRuntime::is_initialized() {
        return Err(EmbassyJoinError::SpawnerNotInitialized);
    }

    let job: Job = Box::pin(future);

    JOB_QUEUE
        .0
        .try_send(job)
        .map_err(|_| EmbassyJoinError::QueueFull)
}
//先检查运行时有没有初始化。
//把任务装箱，变成统一的 Job。
//尝试放入全局任务队列。
//如果队列满了，返回 QueueFull。
impl RuntimeSpec for EmbassyRuntime {
    type JoinHandle<T: MaybeSend + 'static> = EmbassyJoinHandle<T>; //告诉hotaru后端使用自定义的JoinHandle类型
    type JoinError = EmbassyJoinError; //告诉hotaru后端使用自定义的JoinError类型

    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + MaybeSend + 'static,
    {
        spawn_task(future).expect("failed to spawn detached embassy task");
    }
    //启动一个后台任务，但不关心返回值。
    //如果提交失败，会直接 panic，因为这种任务没有地方返回错误。
    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
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

        match spawn_task(task) {
            Ok(()) => EmbassyJoinHandle::waiting(signal),
            Err(error) => EmbassyJoinHandle::failed(error),
        }
    }
    //启动一个有返回值的任务
    //创建一个 Signal。
    //任务执行完成后，把结果发进 Signal。
    //返回一个 EmbassyJoinHandle。
    //调用者等待这个 handle，就能拿到结果。
    type Instant = embassy_time::Instant;
    type TimeoutError = EmbassyTimeoutError;

    fn now() -> Self::Instant {
        embassy_time::Instant::now()
    }

    fn instant_plus(instant: Self::Instant, dur: Duration) -> Self::Instant {
        instant.saturating_add(to_embassy_duration(dur))
    }

    fn sleep(dur: Duration) -> impl Future<Output = ()> + MaybeSend + 'static {
        embassy_time::Timer::after(to_embassy_duration(dur))
    }

    fn sleep_until(deadline: Self::Instant) -> impl Future<Output = ()> + MaybeSend + 'static {
        embassy_time::Timer::at(deadline)
    }
    //这些把 Hotaru 的时间操作接到 Embassy 的时间系统：
    //now：当前时间点。
    //instant_plus：某个时间点加上一段时间。
    //sleep：睡一段时间。
    //sleep_until：睡到某个时间点。
    async fn timeout<F>(dur: Duration, future: F) -> Result<F::Output, Self::TimeoutError>
    where
        F: Future + MaybeSend,
        F::Output: MaybeSend,
    {
        embassy_time::with_timeout(to_embassy_duration(dur), future)
            .await
            .map_err(|_| EmbassyTimeoutError)
    }
    //意思是：给一个任务限定时间。
    //如果任务在规定时间内完成，返回任务结果。
    //如果时间到了还没完成，返回 EmbassyTimeoutError。
    async fn select2<A, B>(a: A, b: B) -> Either<A::Output, B::Output>
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
    //同时等待两个任务，谁先完成就返回谁的结果。
    //第一个先完成：返回 Either::Left
    //第二个先完成：返回 Either::Right
    type OnceCell<T: MaybeSend + Sync + 'static> = EmbassyOnceCell<T>;
    type AsyncMutex<T: MaybeSend + 'static> = EmbassyMutex<T>;
}
//一次性初始化容器用 EmbassyOnceCell。
//异步锁用 EmbassyMutex。
