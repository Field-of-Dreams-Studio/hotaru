use core::future::Future;

use alloc::boxed::Box;

use crate::app::runtime::spec::{BoxFuture, OnceCellCap};

/// Newtype wrapper over `tokio::sync::OnceCell` so we can implement the
/// foreign `OnceCellCap` trait cleanly (the orphan rule blocks implementing
/// it on `tokio::sync::OnceCell<T>` directly).
#[derive(Debug)]
pub struct TokioOnceCell<T>(pub tokio::sync::OnceCell<T>);

impl<T> Default for TokioOnceCell<T> {
    fn default() -> Self {
        Self(tokio::sync::OnceCell::new())
    }
}

impl<T: Send + Sync + 'static> OnceCellCap<T> for TokioOnceCell<T> {
    fn get(&self) -> Option<&T> {
        self.0.get()
    }

    fn get_or_try_init<'a, F, Fut, E>(&'a self, init: F) -> BoxFuture<'a, Result<&'a T, E>>
    where
        F: FnOnce() -> Fut + Send + 'a,
        Fut: Future<Output = Result<T, E>> + Send + 'a,
        E: Send + 'a,
    {
        // Box::pin is the workaround for rustc issue #100013 (HRTB + Send).
        Box::pin(self.0.get_or_try_init(init))
    }
}
