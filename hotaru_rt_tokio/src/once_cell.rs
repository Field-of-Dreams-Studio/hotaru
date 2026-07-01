use core::future::Future;

use hotaru_core::{
    app::runtime::{BoxFuture, OnceCellCap},
    marker::MaybeSend,
};

/// Newtype wrapper over `tokio::sync::OnceCell` so we can implement the
/// Hotaru runtime capability trait without violating orphan rules.
pub struct TokioOnceCell<T>(pub tokio::sync::OnceCell<T>);

impl<T> Default for TokioOnceCell<T> {
    fn default() -> Self {
        Self(tokio::sync::OnceCell::new())
    }
}

impl<T: MaybeSend + Sync + 'static> OnceCellCap<T> for TokioOnceCell<T> {
    fn get(&self) -> Option<&T> {
        self.0.get()
    }

    fn get_or_try_init<'a, F, Fut, E>(&'a self, init: F) -> BoxFuture<'a, Result<&'a T, E>>
    where
        F: FnOnce() -> Fut + MaybeSend + 'a,
        Fut: Future<Output = Result<T, E>> + MaybeSend + 'a,
        E: MaybeSend + 'a,
    {
        Box::pin(async move { self.0.get_or_try_init(init).await })
    }
}
