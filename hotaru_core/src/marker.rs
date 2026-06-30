//! Marker traits and type aliases for Hotaru core.
//!
//! Centralized marker traits and aliases used throughout the framework. All
//! internal code should import universal markers from this module so runtime,
//! IO, and sync code share the same conditional bounds.

use core::future::Future;
use core::pin::Pin;

use alloc::boxed::Box;

// ============ Concurrency Primitives ============

/// Read-write lock alias. `parking_lot::RwLock` (default) or `spin::RwLock`
/// (under `feature = "embedded"`). Both backends are poison-free and return
/// guards directly from `read()`/`write()` — no `unwrap` needed.
#[cfg(feature = "std")]
pub use parking_lot::RwLock as PRwLock;
#[cfg(feature = "embedded")]
pub use spin::RwLock as PRwLock;

/// Read guard for [`PRwLock`].
#[cfg(feature = "std")]
pub use parking_lot::RwLockReadGuard as PRwLockReadGuard;
#[cfg(feature = "embedded")]
pub use spin::RwLockReadGuard as PRwLockReadGuard;

/// Write guard for [`PRwLock`].
#[cfg(feature = "std")]
pub use parking_lot::RwLockWriteGuard as PRwLockWriteGuard;
#[cfg(feature = "embedded")]
pub use spin::RwLockWriteGuard as PRwLockWriteGuard;

/// Mutex alias. `parking_lot::Mutex` (default) or `spin::Mutex` (under
/// `feature = "embedded"`). Both poison-free.
#[cfg(feature = "std")]
pub use parking_lot::Mutex as PMutex;
#[cfg(feature = "embedded")]
pub use spin::Mutex as PMutex;

/// Mutex guard for [`PMutex`].
#[cfg(feature = "std")]
pub use parking_lot::MutexGuard as PMutexGuard;
#[cfg(feature = "embedded")]
pub use spin::MutexGuard as PMutexGuard;

// ============ Future Extensions ============

/// Alias for `Send` on `std` targets; no-op marker on `embedded` targets.
#[cfg(feature = "std")]
pub use core::marker::Send as MaybeSend;

/// Conditional send marker used by futures and shared framework types.
/// `std` builds require real `Send`; `embedded` builds implement this for all
/// types so single-threaded runtimes can keep local futures.
#[cfg(feature = "embedded")]
pub trait MaybeSend {}

#[cfg(feature = "embedded")]
impl<T: ?Sized> MaybeSend for T {}

/// Object-safe helper for boxed runtime futures.
/// It combines `Future + MaybeSend` into one trait-object base, which works
/// even when `MaybeSend` is the embedded no-op marker.
pub trait MaybeSendFuture<T>: Future<Output = T> + MaybeSend {}

impl<T, F> MaybeSendFuture<T> for F where F: Future<Output = T> + MaybeSend {}

/// Boxed runtime future used where returned futures borrow init state.
/// `MaybeSend` is `Send` on `std` and relaxed on `embedded`.
pub type BoxFuture<'a, T> = Pin<Box<dyn MaybeSendFuture<T> + 'a>>;
