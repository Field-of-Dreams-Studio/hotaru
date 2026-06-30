//! Reader and writer traits for Hotaru async IO.
//!
//! `std` is not an IO backend here: blocking `std::io::Read` / `Write`
//! does not fit this async trait surface. Concrete backend adapters live in
//! sibling crates such as `hotaru_io_tokio`, `hotaru_io_futures`, and
//! `hotaru_io_embedded`.

pub mod error;
pub mod read;
pub mod write;

pub use error::HotaruIOError;
pub use read::{HotaruBufRead, HotaruRead};
pub use write::{HotaruBufWrite, HotaruWrite};
