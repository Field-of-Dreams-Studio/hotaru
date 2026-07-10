use hotaru_core::{app::runtime::AsyncMutexCap, marker::MaybeSend};

use embassy_sync::blocking_mutex::raw::RawMutex;

#[cfg(feature = "spawn_send")]
pub type EmbassyRawMutex = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "spawn_local")]
pub type EmbassyRawMutex = embassy_sync::blocking_mutex::raw::NoopRawMutex;

/// Newtype wrapper over `embassy_sync::mutex::Mutex`.
///
/// The raw mutex defaults to [`EmbassyRawMutex`], which follows Hotaru's active
/// `spawn_send` / `spawn_local` feature choice. Generated runtimes may select a
/// different Embassy raw mutex through `define_runtime_worker_pool!(..., raw_mutex = Type)`.
pub struct EmbassyMutex<T, M = EmbassyRawMutex>
where
    M: RawMutex,
{
    pub(crate) inner: embassy_sync::mutex::Mutex<M, T>,
}

// SAFETY: Hotaru requires runtime mutex types to be `Sync`. Under
// `spawn_local`, this wrapper is valid only for Embassy's single-executor
// contract: values must not be shared with another executor or interrupt
// context while protected by a non-Sync raw mutex such as `NoopRawMutex`.
#[cfg(feature = "spawn_local")]
unsafe impl<T, M> Sync for EmbassyMutex<T, M>
where
    T: MaybeSend + 'static,
    M: RawMutex + 'static,
{
}

impl<T, M> EmbassyMutex<T, M>
where
    M: RawMutex,
{
    /// Creates a new async mutex protecting `value`.
    pub fn new(value: T) -> Self {
        Self {
            inner: embassy_sync::mutex::Mutex::new(value),
        }
    }
}

#[cfg(feature = "spawn_send")]
impl<T, M> AsyncMutexCap<T> for EmbassyMutex<T, M>
where
    T: MaybeSend + 'static,
    M: RawMutex + Send + Sync + 'static,
{
    type Guard<'a>
        = embassy_sync::mutex::MutexGuard<'a, M, T>
    where
        Self: 'a;

    fn new(value: T) -> Self {
        Self::new(value)
    }

    async fn lock(&self) -> Self::Guard<'_> {
        self.inner.lock().await
    }
}

#[cfg(feature = "spawn_local")]
impl<T, M> AsyncMutexCap<T> for EmbassyMutex<T, M>
where
    T: MaybeSend + 'static,
    M: RawMutex + 'static,
{
    type Guard<'a>
        = embassy_sync::mutex::MutexGuard<'a, M, T>
    where
        Self: 'a;

    fn new(value: T) -> Self {
        Self::new(value)
    }

    async fn lock(&self) -> Self::Guard<'_> {
        self.inner.lock().await
    }
}
