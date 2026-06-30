// Pull `alloc` into scope so source files can `use alloc::sync::Arc;`
// regardless of std/no_std mode. Harmless in std builds (std already links
// alloc); required once the crate flips to no_std.
extern crate alloc;

#[cfg(all(feature = "std", feature = "embedded"))]
compile_error!("hotaru_core: features `std` and `embedded` are mutually exclusive");

#[cfg(not(any(feature = "std", feature = "embedded")))]
compile_error!("hotaru_core: one of `std` or `embedded` must be enabled");

// Axis 3 is additive: one or more runtime backends may be enabled, and each
// `App`/`Server`/`Client` picks its concrete `Rt`. Only require at least one.
#[cfg(not(any(feature = "rt_tokio", feature = "rt_embassy")))]
compile_error!(
    "hotaru_core: enable at least one async-runtime feature (`rt_tokio` or `rt_embassy`)"
);

#[cfg(all(feature = "spawn_send", feature = "spawn_local"))]
compile_error!("hotaru_core: features `spawn_send` and `spawn_local` are mutually exclusive");

#[cfg(not(any(feature = "spawn_send", feature = "spawn_local")))]
compile_error!(
    "hotaru_core: enable exactly one task-mobility feature (`spawn_send` or `spawn_local`)"
);

#[cfg(all(feature = "rt_tokio", feature = "spawn_local"))]
compile_error!("hotaru_core: `rt_tokio` requires `spawn_send`, not `spawn_local`");

#[cfg(not(any(feature = "io_futures", feature = "io_embedded", feature = "io_tokio")))]
compile_error!("hotaru_core: at least one io_* feature must be enabled");

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
