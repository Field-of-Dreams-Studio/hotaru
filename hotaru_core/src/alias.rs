//! Type aliases for Hotaru core.
//!
//! Centralized type aliases used throughout the framework. All internal code
//! MUST import from this module so the sync backend can be swapped per build.
//!
//! ## Concurrency Primitives
//!
//! Backend is selected by the mutually-exclusive target-flavour features
//! `std` (pulls `parking_lot` — default on host targets, poison-free, fast,
//! small) or `embedded` (pulls `spin` — no_std-friendly, poison-free,
//! alloc-free). Exactly one must be enabled; `lib.rs` enforces this with
//! `compile_error!`. **Spinlock semantics under `embedded`**: fine for short
//! critical sections, bad under contention or on RTOS targets with priority
//! inversion.
//!
//! ```rust
//! use hotaru_core::alias::PRwLock;
//! let lock = PRwLock::new(vec![1, 2, 3]);
//! let guard = lock.read(); // No unwrap needed (poison-free in both backends).
//! assert_eq!(guard[0], 1);
//! ```

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
// Add more type aliases here as needed
