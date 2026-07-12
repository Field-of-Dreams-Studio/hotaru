//! Marker traits and type aliases for Hotaru core.
//!
//! Centralized marker traits and aliases used throughout the framework. All
//! internal code should import universal markers from this module so runtime,
//! IO, and sync code share the same conditional bounds.

use core::future::Future;
use core::pin::Pin;

use alloc::boxed::Box;

#[cfg(all(not(feature = "std"), not(feature = "spin")))]
use core::cell::{Ref, RefCell, RefMut};

// ============ Shared Pointer ============

/// Shared pointer alias used throughout Hotaru core.
///
/// Normal shared builds use `Arc`. No-atomic local builds use `Rc`, because
/// that mode intentionally avoids atomic shared ownership.
#[cfg(not(feature = "spawn_local_no_atomic"))]
pub use alloc::sync::Arc;
#[cfg(feature = "spawn_local_no_atomic")]
pub use alloc::rc::Rc as Arc;

// ============ Concurrency Primitives ============

/// Read-write lock alias. Three backends, chosen by feature availability:
/// `parking_lot::RwLock` on `std`, `spin::RwLock` when `spin` is enabled,
/// and a `RefCell` wrapper when neither lock crate is selected. All three are
/// poison-free and return guards directly from `read()`/`write()` — no
/// `unwrap` needed.
#[cfg(feature = "std")]
pub use parking_lot::RwLock as PRwLock;
#[cfg(all(not(feature = "std"), feature = "spin"))]
pub use spin::RwLock as PRwLock;

/// `RefCell`-backed read-write lock for local/no-atomic builds.
#[cfg(all(not(feature = "std"), not(feature = "spin")))]
pub struct PRwLock<T>(RefCell<T>);

#[cfg(all(not(feature = "std"), not(feature = "spin")))]
impl<T> PRwLock<T> {
    /// Creates a new lock protecting `value`.
    pub fn new(value: T) -> Self {
        Self(RefCell::new(value))
    }

    /// Acquires shared read access. Panics if a write guard is live.
    pub fn read(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    /// Acquires exclusive write access. Panics if any guard is live.
    pub fn write(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}

/// Read guard for [`PRwLock`].
#[cfg(feature = "std")]
pub use parking_lot::RwLockReadGuard as PRwLockReadGuard;
#[cfg(all(not(feature = "std"), feature = "spin"))]
pub use spin::RwLockReadGuard as PRwLockReadGuard;
#[cfg(all(not(feature = "std"), not(feature = "spin")))]
pub type PRwLockReadGuard<'a, T> = Ref<'a, T>;

/// Write guard for [`PRwLock`].
#[cfg(feature = "std")]
pub use parking_lot::RwLockWriteGuard as PRwLockWriteGuard;
#[cfg(all(not(feature = "std"), feature = "spin"))]
pub use spin::RwLockWriteGuard as PRwLockWriteGuard;
#[cfg(all(not(feature = "std"), not(feature = "spin")))]
pub type PRwLockWriteGuard<'a, T> = RefMut<'a, T>;

/// Mutex alias. `parking_lot::Mutex` on `std`, `spin::Mutex` when `spin` is
/// enabled, and a `RefCell` wrapper when neither lock crate is selected. All
/// poison-free.
#[cfg(feature = "std")]
pub use parking_lot::Mutex as PMutex;
#[cfg(all(not(feature = "std"), feature = "spin"))]
pub use spin::Mutex as PMutex;

/// `RefCell`-backed mutex for local/no-atomic builds.
#[cfg(all(not(feature = "std"), not(feature = "spin")))]
pub struct PMutex<T>(RefCell<T>);

#[cfg(all(not(feature = "std"), not(feature = "spin")))]
impl<T> PMutex<T> {
    /// Creates a new mutex protecting `value`.
    pub fn new(value: T) -> Self {
        Self(RefCell::new(value))
    }

    /// Locks the mutex. Panics if a guard is already live.
    pub fn lock(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}

/// Mutex guard for [`PMutex`].
#[cfg(feature = "std")]
pub use parking_lot::MutexGuard as PMutexGuard;
#[cfg(all(not(feature = "std"), feature = "spin"))]
pub use spin::MutexGuard as PMutexGuard;
#[cfg(all(not(feature = "std"), not(feature = "spin")))]
pub type PMutexGuard<'a, T> = RefMut<'a, T>;

// ============ Future Extensions ============

/// Alias for `Send` when spawned tasks may move between execution contexts.
#[cfg(feature = "spawn_send")]
pub use core::marker::Send as MaybeSend;

/// Conditional send marker used by futures and shared framework types.
/// `spawn_send` builds require real `Send`; `spawn_local` builds implement
/// this for all types so local/single-executor runtimes can keep `!Send`
/// futures. This is a task-mobility axis, not a `std`/`no_std` axis.
#[cfg(feature = "spawn_local")]
pub trait MaybeSend {}

#[cfg(feature = "spawn_local")]
impl<T: ?Sized> MaybeSend for T {}

/// Alias for `Sync` when shared values may cross execution contexts.
#[cfg(feature = "spawn_send")]
pub use core::marker::Sync as MaybeSync;

/// Conditional sync marker used by shared framework types.
/// Local/no-atomic builds implement this for all types.
#[cfg(feature = "spawn_local")]
pub trait MaybeSync {}

#[cfg(feature = "spawn_local")]
impl<T: ?Sized> MaybeSync for T {}

/// Convenience marker for values that need both Hotaru mobility markers.
pub trait MaybeSendSync: MaybeSend + MaybeSync {}

impl<T: ?Sized> MaybeSendSync for T where T: MaybeSend + MaybeSync {}

/// Object-safe helper for boxed runtime futures.
/// It combines `Future + MaybeSend` into one trait-object base, which works
/// even when `MaybeSend` is the embedded no-op marker.
pub trait MaybeSendFuture<T>: Future<Output = T> + MaybeSend {}

impl<T, F> MaybeSendFuture<T> for F where F: Future<Output = T> + MaybeSend {}

/// Boxed dyn-future whose `Send`-ness is conditional on the active target.
///
/// `dyn Future + MaybeSend` is rejected on the embedded flavour because
/// `MaybeSend` is a non-auto trait there (E0225 — only auto traits can
/// combine with a non-auto trait in a `dyn` position). This alias picks the
/// right `dyn` shape per target:
///
/// - `spawn_send`: `Pin<Box<dyn Future<Output = T> + Send + 'a>>`
/// - `spawn_local`: `Pin<Box<dyn Future<Output = T> + 'a>>`
///
/// Use it at framework boundaries that erase futures into `Box<dyn>` for
/// later spawning (e.g. `ProtocolEntryTrait::serve`).
#[cfg(feature = "spawn_send")]
pub type MaybeSendBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
#[cfg(feature = "spawn_local")]
pub type MaybeSendBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Boxed runtime future used where returned futures borrow init state.
/// Uses the same conditional dyn shape as [`MaybeSendBoxFuture`].
pub type BoxFuture<'a, T> = MaybeSendBoxFuture<'a, T>;
