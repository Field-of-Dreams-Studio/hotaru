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

// Platform/sync selection is a global mode, not an additive capability.
// If both are enabled, fail early instead of silently choosing between
// `parking_lot`/std assumptions and `spin`/embedded assumptions.
#[cfg(all(feature = "std", feature = "embedded"))]
compile_error!("hotaru_core: features `std` and `embedded` are mutually exclusive");

#[cfg(not(any(feature = "std", feature = "embedded")))]
compile_error!("hotaru_core: one of `std` or `embedded` must be enabled");

// Task-mobility selection is a global mode. `spawn_send` requires movable
// `Send` futures, while `spawn_local` permits local `!Send` futures; choosing
// one silently would hide backend/runtime configuration mistakes.
#[cfg(all(feature = "spawn_send", feature = "spawn_local"))]
compile_error!("hotaru_core: features `spawn_send` and `spawn_local` are mutually exclusive");

#[cfg(not(any(feature = "spawn_send", feature = "spawn_local")))]
compile_error!(
    "hotaru_core: enable exactly one task-mobility feature (`spawn_send` or `spawn_local`)"
);

// `spawn_local_atomic` / `spawn_local_no_atomic` are refinements of
// `spawn_local` (each enables it), so the check above already governs the
// spawn_send-vs-local axis. They differ only in atomic-CAS availability and
// must not both be active at once.
#[cfg(all(feature = "spawn_local_atomic", feature = "spawn_local_no_atomic"))]
compile_error!(
    "hotaru_core: features `spawn_local_atomic` and `spawn_local_no_atomic` are mutually exclusive"
);

/// Crate-internal prelude.
///
/// This is Hotaru's facade for common allocation-backed types and marker
/// aliases. Both `std` and `embedded` builds link `alloc`, so the basic
/// allocation types come directly from `alloc` in all modes. Only aliases with
/// real capability differences (for example [`Arc`](crate::marker::Arc) under
/// `spawn_local_no_atomic`) are cfg-selected behind [`marker`](crate::marker).
pub mod prelude {
    pub use akari::hash::HashMap;
    pub use alloc::boxed::Box;
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;

    pub use crate::marker::{
        Arc, BoxFuture, MaybeSend, MaybeSendBoxFuture, MaybeSendFuture, MaybeSendSync, MaybeSync,
        PMutex, PMutexGuard, PRwLock, PRwLockReadGuard, PRwLockWriteGuard,
    };
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
    Arc, BoxFuture, MaybeSend, MaybeSendBoxFuture, MaybeSendFuture, MaybeSendSync, MaybeSync,
    PRwLock, PRwLockReadGuard, PRwLockWriteGuard,
};

// Helpers `hotaru_core::app::server::run_server*` are the runtime plumbing
// behind the `run_server!` / `run_server_until!` / `run_server_no_block!` /
// `run_server_no_block_until!` proc-macros in `hotaru_trans`.
