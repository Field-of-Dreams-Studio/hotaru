//! Core protocol, routing, connection, and runtime abstractions for Hotaru.
//!
//! This crate is transport- and runtime-neutral. Higher-level crates provide
//! concrete HTTP, Tokio, futures-io, or embedded adapters.

// Pull `alloc` into scope so source files can `use alloc::sync::Arc;`
// regardless of std/no_std mode. Harmless in std builds (std already links
// alloc); required once the crate flips to no_std.
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
