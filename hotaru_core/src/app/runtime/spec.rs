//! Runtime backend abstraction.
//!
//! Everything a runtime needs to satisfy: [`RuntimeSpec`] (spawn / time /
//! select / cap-type projections), [`OnceCellCap`] (async one-time init),
//! [`AsyncMutexCap`] (locks held across `.await`), and [`Either`] (select
//! outcome).
//!
//! # No worked example here — see `hotaru_rt_tokio`
//!
//! `hotaru_core` intentionally ships no concrete `RuntimeSpec` impl.
//! Read `hotaru_rt_tokio` (`TokioRuntime`, `TokioOnceCell`, `TokioMutex`)
//! as the canonical implementation and mirror its structure when adding
//! a new backend. That crate maps cleanly onto the three cap traits here:
//! one file per associated-type wrapper plus one `runtime.rs` binding
//! everything to `RuntimeSpec`.

mod mutex;
mod once_cell;
mod runtime;
mod select;

pub use crate::marker::{BoxFuture, MaybeSendFuture};
pub use mutex::AsyncMutexCap;
pub use once_cell::OnceCellCap;
pub use runtime::RuntimeSpec;
pub use select::Either;
