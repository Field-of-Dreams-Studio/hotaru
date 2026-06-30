use core::future::Future;

/// Async runtime backend abstraction.
///
/// A single trait that carries both spawn shapes:
///
/// * [`spawn_detached`](RuntimeSpec::spawn_detached) — fire-and-forget; the
///   spawned future runs for its side effects and no handle is returned.
/// * [`spawn`](RuntimeSpec::spawn) — returns a typed [`JoinHandle`] for the
///   spawned future's output.
///
/// Stage 6 ships the minimal viable surface that matches the existing
/// tokio-direct behaviour exactly (`+ Send + 'static` bounds); Stage 7 adds
/// sync/time/select methods and Stage 8 relaxes the future bounds to
/// `MaybeSend`.
pub trait RuntimeSpec: 'static {
    /// Join handle produced by [`spawn`](RuntimeSpec::spawn). Awaitable;
    /// yields `Result<T, JoinError>` on completion.
    type JoinHandle<T: Send + 'static>: Future<Output = Result<T, Self::JoinError>>
        + Send
        + 'static;

    /// Error returned when a spawned task ends abnormally (panic / cancel).
    type JoinError: core::error::Error + Send + Sync + 'static;

    /// Detached spawn. The spawned future runs for its side effects; no
    /// handle is returned, no result is observable to the caller.
    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static;

    /// Spawn `future` on the runtime and return a typed join handle.
    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}
