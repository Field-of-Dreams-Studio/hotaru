use alloc::boxed::Box;
use core::future::Future;

use hotaru_core::{
    app::runtime::{BoxFuture, OnceCellCap},
    marker::MaybeSend,
};

use crate::EmbassyMutex;

/// Fallible async once-cell built from Embassy primitives.
///
/// `embassy_sync::once_lock::OnceLock` provides the stored reference while the
/// async mutex serializes fallible initializers so failed attempts leave the
/// cell empty and retryable.
pub struct EmbassyOnceCell<T> {
    value: embassy_sync::once_lock::OnceLock<T>,
    init_lock: EmbassyMutex<()>,
}

impl<T> Default for EmbassyOnceCell<T> {
    fn default() -> Self {
        Self {
            value: embassy_sync::once_lock::OnceLock::new(),
            init_lock: EmbassyMutex::new(()),
        }
    }
}

impl<T: MaybeSend + Sync + 'static> OnceCellCap<T> for EmbassyOnceCell<T> {
    fn get(&self) -> Option<&T> {
        self.value.try_get()
    }

    fn get_or_try_init<'a, F, Fut, E>(&'a self, init: F) -> BoxFuture<'a, Result<&'a T, E>>
    where
        F: FnOnce() -> Fut + MaybeSend + 'a,
        Fut: Future<Output = Result<T, E>> + MaybeSend + 'a,
        E: MaybeSend + 'a,
    {
        Box::pin(async move {
            if let Some(value) = self.value.try_get() {
                return Ok(value);
            }

            let _guard = self.init_lock.0.lock().await;

            if let Some(value) = self.value.try_get() {
                return Ok(value);
            }

            let value = init().await?;
            let _ = self.value.init(value);

            Ok(self
                .value
                .try_get()
                .expect("embassy once cell was initialized"))
        })
    }
}
