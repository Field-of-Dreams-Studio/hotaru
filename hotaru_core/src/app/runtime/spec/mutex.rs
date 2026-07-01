use core::future::Future;

use crate::marker::MaybeSend;

/// Backend-neutral async mutex.
/// Use it only when exclusive mutable access may need to be held across
/// `.await`.
pub trait AsyncMutexCap<T: MaybeSend + 'static>: MaybeSend + Sync + 'static {
    /// Guard type returned by `lock`; deref-coerces to `&mut T`.
    type Guard<'a>: core::ops::DerefMut<Target = T> + MaybeSend + 'a
    where
        Self: 'a;

    /// Creates a new async mutex protecting `value`.
    fn new(value: T) -> Self;

    /// Acquires the mutex and returns its async guard.
    fn lock(&self) -> impl Future<Output = Self::Guard<'_>> + MaybeSend + '_;
}

// TODO(runtime-sync): Consider an `AsyncRwLockCap` only after identifying a
// read-heavy shared-state path where read/write splitting beats mutex
// simplicity across both Tokio and Embassy backends.
