//! Type aliases for Hotaru core.
//!
//! This module provides centralized type aliases used throughout the Hotaru framework.
//! All internal code MUST import from this module to ensure consistency.
//!
//! ## Concurrency Primitives
//!
//! We use `parking_lot` for all synchronization primitives due to:
//! - **No lock poisoning**: Panics don't poison the lock (safer under panic)
//! - **Better performance**: 2-10x faster than std::sync
//! - **Smaller memory footprint**: 1 byte vs 16-24 bytes for std::sync::RwLock
//!
//! ### Usage
//!
//! ```rust
//! use hotaru_core::alias::PRwLock;
//!
//! let lock = PRwLock::new(vec![1, 2, 3]);
//! let guard = lock.read();  // No unwrap needed!
//! assert_eq!(guard[0], 1);
//! ```

// ============ Concurrency Primitives ============

/// Priority-aware RwLock (parking_lot implementation).
///
/// This is an alias to `parking_lot::RwLock` which provides:
/// - **No poisoning**: Panics don't poison the lock
/// - **Better performance**: 2-10x faster than `std::sync::RwLock`
/// - **Smaller size**: 1 byte vs 16-24 bytes
///
/// Use this instead of `std::sync::RwLock` in all Hotaru core code.
///
/// # Example
///
/// ```rust
/// use hotaru_core::alias::PRwLock;
///
/// let data = PRwLock::new(42);
///
/// // Read access (no unwrap needed!)
/// let r = data.read();
/// assert_eq!(*r, 42);
/// drop(r);
///
/// // Write access (no unwrap needed!)
/// let mut w = data.write();
/// *w = 100;
/// ```
pub use parking_lot::RwLock as PRwLock;

/// Read guard for [`PRwLock`].
///
/// This guard is returned by [`PRwLock::read()`] and provides read-only access
/// to the data protected by the lock.
pub use parking_lot::RwLockReadGuard as PRwLockReadGuard;

/// Write guard for [`PRwLock`].
///
/// This guard is returned by [`PRwLock::write()`] and provides mutable access
/// to the data protected by the lock.
pub use parking_lot::RwLockWriteGuard as PRwLockWriteGuard;

/// Priority-aware Mutex (parking_lot implementation).
///
/// Similar to [`PRwLock`], this never poisons and performs better
/// than `std::sync::Mutex`.
///
/// # Example
///
/// ```rust
/// use hotaru_core::alias::PMutex;
///
/// let data = PMutex::new(vec![1, 2, 3]);
///
/// let mut guard = data.lock();  // No unwrap needed!
/// guard.push(4);
/// ```
pub use parking_lot::Mutex as PMutex;

/// Mutex guard for [`PMutex`].
pub use parking_lot::MutexGuard as PMutexGuard;

// ============ Future Extensions ============
// Add more type aliases here as needed
