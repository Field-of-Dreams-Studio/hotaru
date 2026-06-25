//! Framework-owned async IO traits.
//!
//! `HotaruRead`, `HotaruWrite`, and `HotaruBufRead` replace `tokio::io::*` in
//! the framework's public trait surface. Feature-gated blanket impls bridge
//! to whichever IO ecosystem the target ships with:
//!
//! - `std` feature: blanket over `tokio::io::AsyncRead`/`AsyncWrite`/
//!   `AsyncBufRead` (current 0.8 behaviour preserved drop-in).
//! - `embedded` feature: blanket over `embedded_io_async::Read`/`Write`/
//!   `BufRead`.
//!
//! The two feature arms are mutually exclusive (enforced in `lib.rs`), so the
//! blanket impls never overlap. 

use core::future::Future;

/// `Send` under `std`, unconstrained under `embedded`. Use on trait return
/// types that must flow through `tokio::spawn` on std but don't need `Send`
/// on single-threaded embassy.
#[cfg(feature = "std")]
pub use core::marker::Send as MaybeSend;

/// `Send` under `std`, unconstrained under `embedded`. Use on trait return
/// types that must flow through `tokio::spawn` on std but don't need `Send`
/// on single-threaded embassy.
#[cfg(feature = "embedded")]
pub trait MaybeSend {}
#[cfg(feature = "embedded")]
impl<T: ?Sized> MaybeSend for T {}

/// Async byte reader.
pub trait HotaruRead {
    /// Concrete error returned by `read`.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Reads bytes into `buf`, returning the number written.
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a;
}

/// Async byte writer.
pub trait HotaruWrite {
    /// Concrete error returned by `write` and `flush`.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Writes bytes from `buf`, returning the number consumed.
    fn write<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a;

    /// Flushes any internally buffered bytes to the sink.
    fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + '_;
}

/// Buffered async byte reader. Carries protocol-detection peeked bytes
/// through `Protocol::open_channel` without leaking `tokio::io::BufReader`.
pub trait HotaruBufRead: HotaruRead {
    /// Returns a slice of the currently buffered bytes, filling the buffer
    /// from the underlying reader if it's empty.
    fn fill_buf<'a>(
        &'a mut self,
    ) -> impl Future<Output = Result<&'a [u8], Self::Error>> + MaybeSend + 'a;

    /// Marks the first `amt` bytes of the internal buffer as consumed so
    /// the next `fill_buf` skips them.
    fn consume(&mut self, amt: usize);
}

// ============================================================================
// Blanket impls — tokio path (`std` feature)
// ============================================================================

#[cfg(feature = "std")]
mod tokio_blanket {
    use super::*;
    use core::pin::Pin;
    use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

    impl<T> HotaruRead for T
    where
        T: AsyncRead + Unpin + Send + ?Sized,
    {
        type Error = std::io::Error;

        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            AsyncReadExt::read(self, buf).await
        }
    }

    impl<T> HotaruWrite for T
    where
        T: AsyncWrite + Unpin + Send + ?Sized,
    {
        type Error = std::io::Error;

        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            AsyncWriteExt::write(self, buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            AsyncWriteExt::flush(self).await
        }
    }

    impl<T> HotaruBufRead for T
    where
        T: AsyncBufRead + AsyncRead + Unpin + Send + ?Sized,
    {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            AsyncBufReadExt::fill_buf(self).await
        }

        fn consume(&mut self, amt: usize) {
            AsyncBufRead::consume(Pin::new(self), amt)
        }
    }
}

// ============================================================================
// Blanket impls — embedded-io-async path (`embedded` feature)
// ============================================================================

#[cfg(feature = "embedded")]
mod embedded_blanket {
    use super::*;

    impl<T> HotaruRead for T
    where
        T: embedded_io_async::Read + ?Sized,
        T::Error: core::error::Error + Send + Sync + 'static,
    {
        type Error = T::Error;

        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            embedded_io_async::Read::read(self, buf).await
        }
    }

    impl<T> HotaruWrite for T
    where
        T: embedded_io_async::Write + ?Sized,
        T::Error: core::error::Error + Send + Sync + 'static,
    {
        type Error = T::Error;

        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            embedded_io_async::Write::write(self, buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            embedded_io_async::Write::flush(self).await
        }
    }

    impl<T> HotaruBufRead for T
    where
        T: embedded_io_async::BufRead + embedded_io_async::Read + ?Sized,
        T::Error: core::error::Error + Send + Sync + 'static,
    {
        async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
            embedded_io_async::BufRead::fill_buf(self).await
        }

        fn consume(&mut self, amt: usize) {
            embedded_io_async::BufRead::consume(self, amt)
        }
    }
}
