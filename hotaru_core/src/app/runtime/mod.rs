//! Runtime backend abstraction.

pub mod spec;
pub use spec::RuntimeSpec;

#[cfg(feature = "rt_tokio")]
pub mod tokio_impl;
#[cfg(feature = "rt_tokio")]
pub use tokio_impl::TokioRuntime;

// #[cfg(feature = "rt_embassy")]
// pub mod embassy_impl;
// #[cfg(feature = "rt_embassy")]
// pub use embassy_impl::EmbassyRuntime;

/// Default runtime alias used by `Server` / `Client` when callers don't
/// specify one.
#[cfg(feature = "rt_tokio")]
pub type DefaultRuntime = TokioRuntime;

// #[cfg(all(not(feature = "rt_tokio"), feature = "rt_embassy"))]
// pub type DefaultRuntime = EmbassyRuntime;
