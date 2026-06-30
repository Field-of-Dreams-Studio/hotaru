//! Runtime backend abstraction.

pub mod spec;
pub use spec::{AsyncMutexCap, BoxFuture, Either, MaybeSendFuture, OnceCellCap, RuntimeSpec};

// Core intentionally does not define or export concrete runtime backends.
// Facade/backend crates such as `hotaru_rt_tokio` choose and implement them.
