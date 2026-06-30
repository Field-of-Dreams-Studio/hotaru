use crate::app::runtime::spec::AsyncMutexCap;

/// Newtype wrapper over `tokio::sync::Mutex` implementing `AsyncMutexCap`.
#[derive(Debug)]
pub struct TokioMutex<T>(pub tokio::sync::Mutex<T>);

impl<T: Send + 'static> AsyncMutexCap<T> for TokioMutex<T> {
    type Guard<'a>
        = tokio::sync::MutexGuard<'a, T>
    where
        Self: 'a;

    fn new(value: T) -> Self {
        Self(tokio::sync::Mutex::new(value))
    }

    async fn lock(&self) -> Self::Guard<'_> {
        self.0.lock().await
    }
}
