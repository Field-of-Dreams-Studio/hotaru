//! Inert test-only runtime for exercising builder / registry code that only
//! needs `Rt: RuntimeSpec` for its type bounds.
//!
//! `PhantomRt::spawn` / `sleep` / `timeout` return pending or trivial futures;
//! nothing actually schedules. Do not use for behaviour that observes runtime
//! side effects.

use core::future::Future;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use crate::marker::{MaybeSend, MaybeSendSync};

use super::spec::{AsyncMutexCap, Either, OnceCellCap, RuntimeSpec};

pub(crate) struct PhantomRt;

#[derive(Debug)]
pub(crate) struct PhantomRtError;

impl core::fmt::Display for PhantomRtError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("inert test runtime")
    }
}

impl core::error::Error for PhantomRtError {}

pub(crate) struct PhantomJoin<T>(PhantomData<fn() -> T>);

impl<T: MaybeSend + 'static> Future for PhantomJoin<T> {
    type Output = Result<T, PhantomRtError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

pub(crate) struct PhantomOnceCell<T>(PhantomData<fn() -> T>);

impl<T> Default for PhantomOnceCell<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: MaybeSendSync + 'static> OnceCellCap<T> for PhantomOnceCell<T> {
    fn get(&self) -> Option<&T> {
        None
    }

    fn get_or_try_init<'a, F, Fut, E>(
        &'a self,
        init: F,
    ) -> crate::marker::MaybeSendBoxFuture<'a, Result<&'a T, E>>
    where
        F: FnOnce() -> Fut + MaybeSend + 'a,
        Fut: Future<Output = Result<T, E>> + MaybeSend + 'a,
        E: MaybeSend + 'a,
    {
        drop(init);
        alloc::boxed::Box::pin(core::future::pending())
    }
}

pub(crate) struct PhantomMutex<T>(PhantomData<fn() -> T>);
pub(crate) struct PhantomGuard<'a, T>(PhantomData<&'a mut T>);

impl<T> Deref for PhantomGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unreachable!("the inert test mutex is never locked")
    }
}

impl<T> DerefMut for PhantomGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unreachable!("the inert test mutex is never locked")
    }
}

impl<T: MaybeSend + 'static> AsyncMutexCap<T> for PhantomMutex<T> {
    type Guard<'a>
        = PhantomGuard<'a, T>
    where
        Self: 'a;

    fn new(_value: T) -> Self {
        Self(PhantomData)
    }

    fn lock(&self) -> impl Future<Output = Self::Guard<'_>> + MaybeSend + '_ {
        core::future::pending()
    }
}

impl RuntimeSpec for PhantomRt {
    type JoinHandle<T: MaybeSend + 'static> = PhantomJoin<T>;
    type JoinError = PhantomRtError;

    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + MaybeSend + 'static,
    {
        drop(future);
    }

    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + MaybeSend + 'static,
        F::Output: MaybeSend + 'static,
    {
        drop(future);
        PhantomJoin(PhantomData)
    }

    type Instant = Duration;
    type TimeoutError = PhantomRtError;

    fn now() -> Self::Instant {
        Duration::ZERO
    }

    fn instant_plus(instant: Self::Instant, duration: Duration) -> Self::Instant {
        instant.saturating_add(duration)
    }

    fn sleep(_duration: Duration) -> impl Future<Output = ()> + MaybeSend + 'static {
        async {}
    }

    fn sleep_until(_deadline: Self::Instant) -> impl Future<Output = ()> + MaybeSend + 'static {
        async {}
    }

    fn timeout<F>(
        _duration: Duration,
        future: F,
    ) -> impl Future<Output = Result<F::Output, Self::TimeoutError>> + MaybeSend
    where
        F: Future + MaybeSend,
        F::Output: MaybeSend,
    {
        async move { Ok(future.await) }
    }

    fn select2<A, B>(a: A, b: B) -> impl Future<Output = Either<A::Output, B::Output>> + MaybeSend
    where
        A: Future + MaybeSend,
        B: Future + MaybeSend,
        A::Output: MaybeSend,
        B::Output: MaybeSend,
    {
        async move {
            let output = a.await;
            drop(b);
            Either::Left(output)
        }
    }

    type OnceCell<T: MaybeSendSync + 'static> = PhantomOnceCell<T>;
    type AsyncMutex<T: MaybeSend + 'static> = PhantomMutex<T>;
}
