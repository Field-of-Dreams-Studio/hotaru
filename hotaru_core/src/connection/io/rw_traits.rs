//! Reader and writer traits for Hotaru async IO.
//!
//! `std` is not an IO backend here: blocking `std::io::Read` / `Write`
//! does not fit this async trait surface. Backend blanket impls come from
//! `tokio` or `embedded`; std-only users implement the Hotaru traits
//! directly or use their default methods.

pub mod error;
pub mod read;
pub mod write;

#[cfg(feature = "tokio")]
mod tokio_blanket;

#[cfg(feature = "embedded")]
mod embedded_blanket;

pub use error::HotaruIOError;
pub use read::{HotaruBufRead, HotaruRead};
pub use write::{HotaruWrite, HotaruBufWrite};

// Model: backend traits -> Hotaru traits; each backend feature gates blanket impls that auto-implements HotaruRead/HotaruWrite/HotaruBufRead for matching backend IO types.
