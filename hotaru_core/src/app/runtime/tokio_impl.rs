use core::future::Future;

use super::RuntimeSpec;

/// Tokio-backed runtime. Spawns via `tokio::spawn`; join handle is
/// `tokio::task::JoinHandle<T>`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TokioRuntime;

impl RuntimeSpec for TokioRuntime {
    type JoinHandle<T: Send + 'static> = tokio::task::JoinHandle<T>;
    type JoinError = tokio::task::JoinError;

    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(future)
    }
} 
