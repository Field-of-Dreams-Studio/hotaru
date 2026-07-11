//! embedded-io-async adapter backend for Hotaru.
//!
//! Mirrors `hotaru_core`: `#![no_std]` whenever the `std` feature is off (i.e.
//! under `feature = "embedded"`). The adapter itself names only `core` prelude
//! items (`Result`, `Into`, `Unpin`, …) and slices, so no `alloc` import is
//! needed here — the `Vec`/`String` used by `HotaruBufRead`'s default methods
//! live in `hotaru_core`.

#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;

#[cfg(feature = "spawn_local")]
use hotaru_core::connection::{
    HotaruBufRead, HotaruBufReader, HotaruBufWriter, HotaruRead, HotaruWrite, MaybeSend,
};

/// Backend tag for `embedded-io-async` IO values.
pub enum EmbeddedBackend {}

/// Error returned by [`EmbeddedIo`] adapters.
///
/// Driver-specific embedded IO errors are normalized to
/// [`embedded_io_async::ErrorKind`]. Hotaru-owned sentinel conditions stay
/// explicit because `embedded-io` has no exact EOF-before-buffer-filled kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedIoError {
    /// Error returned by the wrapped embedded IO value.
    Backend(embedded_io_async::ErrorKind),
    /// Reader returned 0 before `read_exact` filled its buffer.
    UnexpectedEof,
    /// Writer accepted 0 bytes before `write_all` drained its buffer.
    WriteZero,
}

impl EmbeddedIoError {
    #[cfg(feature = "spawn_local")]
    fn from_backend<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        Self::Backend(error.kind())
    }

    /// Returns the closest standard embedded IO error kind for this error.
    pub fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::Backend(kind) => *kind,
            Self::UnexpectedEof => embedded_io_async::ErrorKind::Other,
            Self::WriteZero => embedded_io_async::ErrorKind::WriteZero,
        }
    }
}

impl fmt::Display for EmbeddedIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(kind) => write!(f, "embedded IO error: {kind:?}"),
            Self::UnexpectedEof => f.write_str("unexpected EOF before buffer was filled"),
            Self::WriteZero => f.write_str("writer accepted 0 bytes"),
        }
    }
}

impl core::error::Error for EmbeddedIoError {}

impl embedded_io_async::Error for EmbeddedIoError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        EmbeddedIoError::kind(self)
    }
}

/// Local adapter for `embedded-io-async` IO sources.
///
/// Embedded IO values opt into Hotaru IO by wrapping in this newtype. The impl
/// target is local, so it cannot overlap with other backend adapters.
pub struct EmbeddedIo<T> {
    inner: T,
}

impl<T> EmbeddedIo<T> {
    /// Wraps an `embedded-io-async` value for use with Hotaru IO traits.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Consumes the adapter and returns the wrapped IO value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Returns a shared reference to the wrapped IO value.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the wrapped IO value.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> From<T> for EmbeddedIo<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

#[cfg(feature = "spawn_local")]
impl<T> HotaruRead for EmbeddedIo<T>
where
    T: embedded_io_async::Read + Unpin + MaybeSend + 'static,
{
    type Error = EmbeddedIoError;
    type Buffered = HotaruBufReader<Self>;

    fn into_buf(self) -> Self::Buffered {
        HotaruBufReader::new(self)
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Read::read(&mut self.inner, buf)
            .await
            .map_err(EmbeddedIoError::from_backend)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Read::read(&mut self.inner, &mut buf[total..])
                .await
                .map_err(EmbeddedIoError::from_backend)?
            {
                0 => return Err(EmbeddedIoError::UnexpectedEof),
                n => total += n,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "spawn_local")]
impl<T> HotaruWrite for EmbeddedIo<T>
where
    T: embedded_io_async::Write + Unpin + MaybeSend + 'static,
{
    type Error = EmbeddedIoError;
    type Buffered = HotaruBufWriter<Self>;

    fn into_buf_write(self) -> Self::Buffered {
        HotaruBufWriter::new(self)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Write::write(&mut self.inner, buf)
            .await
            .map_err(EmbeddedIoError::from_backend)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io_async::Write::flush(&mut self.inner)
            .await
            .map_err(EmbeddedIoError::from_backend)
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Write::write(&mut self.inner, &buf[total..])
                .await
                .map_err(EmbeddedIoError::from_backend)?
            {
                0 => return Err(EmbeddedIoError::WriteZero),
                n => total += n,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "spawn_local")]
impl<T> HotaruBufRead for EmbeddedIo<T>
where
    T: embedded_io_async::BufRead + embedded_io_async::Read + Unpin + MaybeSend + 'static,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        embedded_io_async::BufRead::fill_buf(&mut self.inner)
            .await
            .map_err(EmbeddedIoError::from_backend)
    }

    fn consume(&mut self, amt: usize) {
        embedded_io_async::BufRead::consume(&mut self.inner, amt)
    }
}
