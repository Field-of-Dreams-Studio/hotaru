use core::future::Future;

use crate::marker::MaybeSend;

use super::BoxFuture;

/// Backend-neutral async one-time-init cell.
/// Used for lazily materialized shared resources such as inbound/outbound
/// transport handles.
pub trait OnceCellCap<T: MaybeSend + Sync + 'static>: Default + MaybeSend + Sync + 'static {
    /// Returns the initialized value, or `None` if not yet initialized.
    fn get(&self) -> Option<&T>;

    /// Initializes the cell from `init` if empty, then returns the value.
    /// `init` is awaited only once across racing callers.
    fn get_or_try_init<'a, F, Fut, E>(&'a self, init: F) -> BoxFuture<'a, Result<&'a T, E>>
    where
        F: FnOnce() -> Fut + MaybeSend + 'a,
        Fut: Future<Output = Result<T, E>> + MaybeSend + 'a,
        E: MaybeSend + 'a;
}
