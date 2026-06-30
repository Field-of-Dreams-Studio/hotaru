//! Runtime backend abstraction.

pub mod spec;
pub use spec::{AsyncMutexCap, BoxFuture, Either, MaybeSendFuture, OnceCellCap, RuntimeSpec};

#[cfg(feature = "rt_tokio")]
pub mod tokio_impl;
#[cfg(feature = "rt_tokio")]
pub use tokio_impl::{TokioMutex, TokioOnceCell, TokioRuntime};

// #[cfg(feature = "rt_embassy")]
// pub mod embassy_impl;
// #[cfg(feature = "rt_embassy")]
// pub use embassy_impl::EmbassyRuntime;

// Core intentionally does not define a default runtime alias. Facade crates
// such as `hotaru` choose their own defaults.
