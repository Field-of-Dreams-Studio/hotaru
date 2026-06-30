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

#[cfg(not(any(feature = "io_futures", feature = "io_embedded", feature = "io_tokio")))]
compile_error!("hotaru_core: at least one io_* feature must be enabled");

// Type aliases (must be declared before other modules that use it)
pub mod alias;

pub mod app;

pub mod executable;

pub mod connection;
pub mod debug;
pub mod protocol;
pub mod url;

pub use akari::*;

// Re-export commonly used type aliases
pub use alias::{PRwLock, PRwLockReadGuard, PRwLockWriteGuard};
