use hotaru_core::{app::runtime::AsyncMutexCap, marker::MaybeSend};

#[cfg(feature = "spawn_send")]
pub(crate) type EmbassyRawMutex = embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "spawn_local")]
pub(crate) type EmbassyRawMutex = embassy_sync::blocking_mutex::raw::NoopRawMutex;

/// Newtype wrapper over `embassy_sync::mutex::Mutex`.
///
/// Under `spawn_local`, this uses Embassy's `NoopRawMutex`, so it inherits the
/// single-executor contract: values must not be accessed from another executor
/// or interrupt context while protected by this mutex.
pub struct EmbassyMutex<T>(pub(crate) embassy_sync::mutex::Mutex<EmbassyRawMutex, T>);

// SAFETY: Hotaru requires runtime mutex types to be `Sync` even for
// `spawn_local`. The local backend uses this only under Embassy's single
// executor contract; `spawn_send` turns `MaybeSend` into real `Send`.
unsafe impl<T: MaybeSend + 'static> Sync for EmbassyMutex<T> {}

impl<T> EmbassyMutex<T> {
    /// Creates a new async mutex protecting `value`.
    pub fn new(value: T) -> Self {
        Self(embassy_sync::mutex::Mutex::new(value))
    }
}

impl<T: MaybeSend + 'static> AsyncMutexCap<T> for EmbassyMutex<T> {
    type Guard<'a>
        = embassy_sync::mutex::MutexGuard<'a, EmbassyRawMutex, T>
    where
        Self: 'a;

    fn new(value: T) -> Self {
        Self::new(value)
    }

    async fn lock(&self) -> Self::Guard<'_> {
        self.0.lock().await
    }
}
