/// Framework-owned IO error. Concrete, non-generic, no-alloc, and identical
/// across feature sets. It starts with only the framework-level sentinel
/// conditions Hotaru itself needs to manufacture.
///
/// Do **not** add a catch-all backend variant up front. When a concrete impl
/// needs to surface another backend failure, add a concrete variant and the
/// corresponding conversion at that impl point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotaruIOError {
    /// Reader returned 0 before `read_exact` filled its buffer.
    UnexpectedEof,
    /// Writer accepted 0 bytes before `write_all` drained its buffer.
    WriteZero,
}

impl core::fmt::Display for HotaruIOError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected EOF before buffer was filled"),
            Self::WriteZero => f.write_str("writer accepted 0 bytes"),
        }
    }
}

impl core::error::Error for HotaruIOError {}
