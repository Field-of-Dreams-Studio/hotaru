//! Runtime backend abstraction.

pub mod spec;
pub use spec::RuntimeSpec;

#[cfg(feature = "rt_tokio")]
pub mod tokio_impl;
#[cfg(feature = "rt_tokio")]
pub use tokio_impl::TokioRuntime;

// `rt_embassy` is a Phase-1 marker in Stage 6.0. The placeholder
// `EmbassyRuntime` implementation lands in Stage 9.C Phase 1, after the
// runtime trait surface has been corrected.

/// Default runtime alias used by `Server` / `Client` when callers don't
/// specify one. Picks tokio under `feature = "rt_tokio"`; falls back to
/// embassy under embassy-only.
#[cfg(feature = "rt_tokio")]
pub type DefaultRuntime = TokioRuntime;
