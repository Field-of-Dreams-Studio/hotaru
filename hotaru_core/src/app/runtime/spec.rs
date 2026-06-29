use core::future::Future;

/// Async runtime backend abstraction.
pub trait RuntimeSpec: 'static {
    /// Join handle produced by [`spawn`]. Awaitable; yields
    /// `Result<T, JoinError>` on completion.
    type JoinHandle<T: Send + 'static>: Future<Output = Result<T, Self::JoinError>>
        + Send
        + 'static;

    /// Error returned when a spawned task ends abnormally (panic / cancel).
    type JoinError: core::error::Error + Send + Sync + 'static;

    /// Spawn `future` on the runtime.
    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
} 
