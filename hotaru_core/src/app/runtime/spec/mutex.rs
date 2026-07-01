use core::future::Future;

use crate::marker::MaybeSend;

/// Backend-neutral async mutex — the "held across `.await`" flavour.
///
/// Use this only when exclusive mutable access must span awaits (e.g. a
/// write half held across a multi-frame protocol send). For short
/// critical sections that don't await inside the lock, prefer the sync
/// [`PMutex`](crate::marker::PMutex) — cheaper, no task switch on
/// contention.
///
/// # Concurrency contract
///
/// - `lock` must be **fair** (queue-based), not opportunistic — callers
///   rely on this to bound worst-case latency under contention.
/// - The returned [`Guard`](AsyncMutexCap::Guard) is `MaybeSend` and may
///   cross `.await` boundaries.
/// - Dropping a `lock()` future before it resolves must cancel the
///   pending lock request cleanly — no phantom waiters, no held lock.
/// - Dropping the guard releases the lock unconditionally; no poisoning
///   on task cancellation.
pub trait AsyncMutexCap<T: MaybeSend + 'static>: MaybeSend + Sync + 'static {
    /// Guard type returned by [`lock`](AsyncMutexCap::lock); deref-coerces
    /// to `&mut T`. Drop the guard to release the lock.
    type Guard<'a>: core::ops::DerefMut<Target = T> + MaybeSend + 'a
    where
        Self: 'a;

    /// Creates a new async mutex protecting `value`. Never blocks.
    fn new(value: T) -> Self;

    /// Acquires the mutex, waiting until it is free. See the trait-level
    /// "Concurrency contract" for fairness and cancel-safety guarantees.
    fn lock(&self) -> impl Future<Output = Self::Guard<'_>> + MaybeSend + '_;
}

// TODO(runtime-sync): Consider an `AsyncRwLockCap` only after identifying a
// read-heavy shared-state path where read/write splitting beats mutex
// simplicity across both Tokio and Embassy backends.
