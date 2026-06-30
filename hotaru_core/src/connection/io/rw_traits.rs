//! Reader and writer traits for Hotaru async IO.
//!
//! `std` is not an IO backend here: blocking `std::io::Read` / `Write`
//! does not fit this async trait surface. Backend blanket impls come from
//! `tokio` or `embedded`; std-only users implement the Hotaru traits
//! directly or use their default methods.

pub mod error;
pub mod read;
pub mod write;

#[cfg(feature = "io_tokio")]
mod tokio_blanket;

#[cfg(feature = "io_embedded")]
mod embedded_blanket;

#[cfg(feature = "io_futures")]
mod futures_blanket;

pub use error::HotaruIOError;
pub use read::{HotaruBufRead, HotaruRead};
pub use write::{HotaruWrite, HotaruBufWrite};

// Backend-tagged IO adapters live alongside each backend's impls.
#[cfg(feature = "io_tokio")]
pub use tokio_blanket::{TokioBackend, TokioIo};

#[cfg(feature = "io_embedded")]
pub use embedded_blanket::{EmbeddedBackend, EmbeddedIo};

#[cfg(feature = "io_futures")]
pub use futures_blanket::{FuturesBackend, FuturesIo};

// Model: backend traits -> Hotaru traits; each backend feature gates blanket impls that auto-implements HotaruRead/HotaruWrite/HotaruBufRead for matching backend IO types.
