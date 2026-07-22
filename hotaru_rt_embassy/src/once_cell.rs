use core::future::Future;

use hotaru_core::{app::runtime::OnceCellCap, prelude::*};

use embassy_sync::blocking_mutex::raw::RawMutex;

use crate::{EmbassyMutex, EmbassyRawMutex};

/// Fallible async once-cell built from Embassy primitives.
///
/// `embassy_sync::once_lock::OnceLock` provides the stored reference while the
/// async mutex serializes fallible initializers so failed attempts leave the
/// cell empty and retryable.
pub struct EmbassyOnceCell<T, M = EmbassyRawMutex>
where
    M: RawMutex,
{
    value: embassy_sync::once_lock::OnceLock<T>,
    init_lock: EmbassyMutex<(), M>,
}

impl<T, M> Default for EmbassyOnceCell<T, M>
where
    M: RawMutex,
{
    fn default() -> Self {
        Self {
            value: embassy_sync::once_lock::OnceLock::new(),
            init_lock: EmbassyMutex::new(()),
        }
    }
}

#[cfg(feature = "spawn_send")]
impl<T, M> OnceCellCap<T> for EmbassyOnceCell<T, M>
where
    T: MaybeSendSync + 'static,
    M: RawMutex + Send + Sync + 'static,
{
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

            let _guard = self.init_lock.inner.lock().await;

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

#[cfg(feature = "spawn_local")]
impl<T, M> OnceCellCap<T> for EmbassyOnceCell<T, M>
where
    T: MaybeSendSync + 'static,
    M: RawMutex + 'static,
{
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

            let _guard = self.init_lock.inner.lock().await;

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
