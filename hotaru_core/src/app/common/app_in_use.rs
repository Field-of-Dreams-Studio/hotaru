use core::fmt;

/// Returned by `try_combine` when an app `Arc` is still shared; gives both
/// apps back so the caller can retry or fall back.
pub struct AppInUse<A> {
    /// The left side, unchanged (reassembled if it had been unwrapped).
    pub app: A,
    /// The untouched right side.
    pub other: A,
}

impl<A> fmt::Debug for AppInUse<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AppInUse")
    }
}

impl<A> fmt::Display for AppInUse<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("app combine refused: an app Arc is still shared")
    }
}

impl<A> core::error::Error for AppInUse<A> {}
