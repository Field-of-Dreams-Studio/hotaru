use core::future::Future;

use super::RuntimeSpec;

/// Capability for runtimes that expose a **sync entry point** — a
/// `block_on` that builds/enters the executor from ordinary sync code
/// and drives one future to completion.
///
/// Tokio implements this by constructing a fresh multi-thread runtime
/// and calling `runtime.block_on(future)`. Smol / async-std do the
/// analogous thing. **Embassy deliberately does not** — embassy's
/// executor is entered through `#[embassy_executor::main]` (or a
/// hand-rolled `Executor::run()` on some ports), not through a sync
/// `block_on` call. Splitting this out as a separate capability keeps
/// the universal `RuntimeSpec` surface embassy-implementable while the
/// blocking macros (`run_server!`, `run_server_until!`) opt into the
/// stricter bound at the call site.
///
/// Framework consumers use this trait through the [`run_server`] /
/// [`run_server_until`] helper functions in
/// [`crate::app::server`] and their macro wrappers.
///
/// [`run_server`]: crate::app::server::run_server
/// [`run_server_until`]: crate::app::server::run_server_until
pub trait BlockingRuntimeCap: RuntimeSpec {
    /// Build the backend's runtime, block the current thread, and drive
    /// `future` to completion. The `+ 'static` bound matches tokio's
    /// `Runtime::block_on` — the future can hold onto owned state
    /// across the whole `block_on` lifetime.
    ///
    /// # Panics
    ///
    /// Backend-defined. Tokio's impl panics if runtime construction
    /// fails (unusual) or if invoked from inside an existing tokio
    /// runtime (nested `block_on` — user is calling this from an async
    /// context by mistake).
    fn block_on<F>(future: F)
    where
        F: Future<Output = ()> + 'static;
}
