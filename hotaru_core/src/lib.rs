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

// Marker traits and type aliases (must be declared before modules that use them).
pub mod marker;

// Backward-compatible alias shim; new code should use `marker`.
pub mod alias;

pub mod app;

pub mod executable;

pub mod connection;
pub mod debug;
pub mod protocol;
pub mod url;

pub use akari::*;

// Re-export commonly used marker aliases.
pub use marker::{
    BoxFuture, MaybeSend, MaybeSendBoxFuture, MaybeSendFuture, PRwLock, PRwLockReadGuard,
    PRwLockWriteGuard,
};
