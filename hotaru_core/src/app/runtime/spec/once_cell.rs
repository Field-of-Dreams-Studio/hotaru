use core::future::Future;

use crate::marker::MaybeSend;

use super::BoxFuture;

/// Backend-neutral async one-time-init cell.
///
/// Used by the framework for lazily materialised shared resources
/// (`Server::inbound`, `Client::outbound`) built once per process and
/// shared read-only thereafter.
///
/// # Concurrency contract
///
/// - `get_or_try_init`: exactly one racing caller's `init` runs; others
///   wait on the winner and observe the same value.
/// - If `init` returns `Err`, the cell stays empty — subsequent calls
///   may retry. Do **not** poison on failure.
/// - Panic behaviour is implementation-defined; callers must not rely on
///   a specific outcome.
/// - `get` never blocks and never triggers init.
///
/// # Why `BoxFuture` (not `impl Future`)
///
/// When a caller's outer `Rt::spawn(async move { ... .await })` tries to
/// prove the body is `Send`, HRTB lifetimes on an `impl Future + Send`
/// projection here fail Send-inference (rustc #100013). Boxing flattens
/// the lifetime structure into a concrete trait object whose Send-ness
/// is captured by the type. Cost: one allocation per init, once per
/// resource lifetime.
pub trait OnceCellCap<T: MaybeSend + Sync + 'static>: Default + MaybeSend + Sync + 'static {
    /// Returns the initialised value, or `None` if the cell is empty.
    /// Never triggers init; never awaits.
    fn get(&self) -> Option<&T>;

    /// Race-safe async init. See the trait-level "Concurrency contract"
    /// for failure and panic semantics.
    fn get_or_try_init<'a, F, Fut, E>(&'a self, init: F) -> BoxFuture<'a, Result<&'a T, E>>
    where
        F: FnOnce() -> Fut + MaybeSend + 'a,
        Fut: Future<Output = Result<T, E>> + MaybeSend + 'a,
        E: MaybeSend + 'a;
}
