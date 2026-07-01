//! Core protocol, routing, connection, and runtime abstractions for Hotaru.
//!
//! This crate is transport- and runtime-neutral. Higher-level crates provide
//! concrete HTTP, Tokio, futures-io, or embedded adapters.
//!
//! # `no_std` under `embedded`
//!
//! When the `std` feature is off (i.e. under `feature = "embedded"`), the
//! crate root declares `#![no_std]`. `alloc` is still available so
//! `String`, `Vec`, `Box`, `format!`, and friends remain usable — only
//! the `std::` prelude items disappear. Source files pull them back in
//! through the crate-internal [`prelude`] module (see below).

#![cfg_attr(not(feature = "std"), no_std)]

// `#[macro_use] extern crate alloc;` brings `format!` and `vec!` into
// every module crate-wide (the macros come from `alloc` under no_std, or
// std's identical re-exports under std). Types like `String`, `Vec`, and
// `Box` still need per-file imports — those come through the
// [`prelude`] module below.
#[macro_use]
extern crate alloc;

#[cfg(all(feature = "std", feature = "embedded"))]
compile_error!("hotaru_core: features `std` and `embedded` are mutually exclusive");

#[cfg(not(any(feature = "std", feature = "embedded")))]
compile_error!("hotaru_core: one of `std` or `embedded` must be enabled");

#[cfg(all(feature = "spawn_send", feature = "spawn_local"))]
compile_error!("hotaru_core: features `spawn_send` and `spawn_local` are mutually exclusive");

#[cfg(not(any(feature = "spawn_send", feature = "spawn_local")))]
compile_error!(
    "hotaru_core: enable exactly one task-mobility feature (`spawn_send` or `spawn_local`)"
);

/// Crate-internal prelude.
///
/// Under `no_std` (`not(feature = "std")`) this re-exports
/// [`akari::prelude`], which supplies `Box`, `String`, `ToString`,
/// `Vec`, `format!`, `vec!`, and `HashMap` — everything that
/// `std::prelude` would otherwise auto-import under std.
///
/// Files bring the prelude in with:
///
/// ```ignore
/// #[cfg(not(feature = "std"))]
/// use crate::prelude::*;
/// ```
///
/// Under `std` the import is elided entirely (std's own prelude covers
/// the same names), so nothing here fires and no "unused import"
/// warnings appear.
///
/// The `akari::prelude` module itself is `#[cfg(feature = "no_std")]`
/// on the akari side, activated for us through the `akari/no_std`
/// entry in the `lite` feature — so this re-export resolves iff both
/// crates are in their no_std flavour.
pub mod prelude {
    #[cfg(not(feature = "std"))]
    pub use akari::prelude::*;
}

/// Shared marker traits and task-mobility aliases.
///
/// Must be declared before modules that use them.
pub mod marker;

/// Backward-compatible alias shim; new code should use [`marker`].
pub mod alias;

/// Application runtimes, builders, server/client types, and runtime traits.
pub mod app;

/// Executable handlers, middleware chains, and protocol entry registries.
pub mod executable;

/// Transport-neutral connection, stream, and async IO traits.
pub mod connection;
/// Debug logging helpers used by Hotaru internals.
pub mod debug;
/// Protocol traits, request contexts, messages, and protocol flow types.
pub mod protocol;
/// URL pattern parsing, routing trees, and path matching.
pub mod url;

pub use akari::*;

// Re-export commonly used marker aliases.
pub use marker::{
    BoxFuture, MaybeSend, MaybeSendBoxFuture, MaybeSendFuture, PRwLock, PRwLockReadGuard,
    PRwLockWriteGuard,
};

// Helpers `hotaru_core::app::server::run_server*` are the runtime plumbing
// behind the `run_server!` / `run_server_until!` / `run_server_no_block!` /
// `run_server_no_block_until!` proc-macros in `hotaru_trans`.
