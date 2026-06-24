// Pull `alloc` into scope so source files can `use alloc::sync::Arc;`
// regardless of std/no_std mode. Harmless in std builds (std already links
// alloc); required once the crate flips to no_std.
extern crate alloc;

// Target-flavour features are mutually exclusive — `std` (host targets,
// pulls parking_lot today + std-flavoured surfaces in future stages) or
// `embedded` (no_std-friendly, pulls spin today). Default features include
// `std`.
#[cfg(all(feature = "std", feature = "embedded"))]
compile_error!("hotaru_core: features `std` and `embedded` are mutually exclusive");

#[cfg(not(any(feature = "std", feature = "embedded")))]
compile_error!("hotaru_core: one of `std` or `embedded` must be enabled");

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
