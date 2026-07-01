//! Reader and writer traits for Hotaru async IO.
//!
//! `std` is not an IO backend here: blocking `std::io::Read` / `Write`
//! does not fit this async trait surface. Concrete backend adapters live in
//! sibling crates such as `hotaru_io_tokio`, `hotaru_io_futures`, and
//! `hotaru_io_embedded`.

/// Shared IO error type for framework-manufactured IO failures.
pub mod error;
/// Async read-side Hotaru IO traits.
pub mod read;
/// Async write-side Hotaru IO traits.
pub mod write;

pub use error::HotaruIOError;
pub use read::{HotaruBufRead, HotaruRead};
pub use write::{HotaruBufWrite, HotaruWrite};
