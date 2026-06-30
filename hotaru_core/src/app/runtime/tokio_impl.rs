use core::future::Future;

use super::RuntimeSpec;

/// Tokio-backed runtime. `spawn_detached` and `spawn` both forward to
/// `tokio::spawn`; the difference is whether the returned
/// `tokio::task::JoinHandle<T>` is exposed.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokioRuntime;

impl RuntimeSpec for TokioRuntime {
    type JoinHandle<T: Send + 'static> = tokio::task::JoinHandle<T>;
    type JoinError = tokio::task::JoinError;

    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        // `tokio::spawn` returns a must_use JoinHandle; drop it explicitly
        // to signal fire-and-forget intent.
        let _ = tokio::spawn(future);
    }

    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(future)
    }
} 
