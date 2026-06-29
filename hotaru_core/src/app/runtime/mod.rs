//! Runtime backend abstraction.

pub mod spec;
pub use spec::RuntimeSpec;

#[cfg(feature = "tokio")]
pub mod tokio_impl;
#[cfg(feature = "tokio")]
pub use tokio_impl::TokioRuntime;

#[cfg(feature = "embassy")]
pub mod embassy_impl;
#[cfg(feature = "embassy")]
pub use embassy_impl::EmbassyRuntime;

/// Default runtime alias used by `Server` / `Client` when callers don't
/// specify one. Picks tokio under `feature = "tokio"`; falls back to
/// embassy under embassy-only.
#[cfg(feature = "tokio")]
pub type DefaultRuntime = TokioRuntime;

#[cfg(all(not(feature = "tokio"), feature = "embassy"))]
pub type DefaultRuntime = EmbassyRuntime; 
