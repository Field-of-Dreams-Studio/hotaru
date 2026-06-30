use core::pin::Pin;

use super::super::IoCompat;
use super::{HotaruBufRead, HotaruBufWrite, HotaruRead, HotaruWrite};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Backend tag for Tokio IO types.
pub enum TokioBackend {}

/// Backend-tagged adapter for Tokio IO sources.
///
/// `io_tokio` also keeps the raw blanket impls below for backwards
/// compatibility, so wrapping in `TokioIo` is optional for Tokio types.
pub type TokioIo<T> = IoCompat<T, TokioBackend>;

impl<T> HotaruRead for T
where
    T: AsyncRead + Unpin + Send + 'static,
{
    type Error = std::io::Error;
    type Buffered = tokio::io::BufReader<Self>;

    fn into_buf(self) -> Self::Buffered {
        tokio::io::BufReader::new(self)
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        AsyncReadExt::read(self, buf).await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        AsyncReadExt::read_exact(self, buf).await.map(|_| ())
    }
}

impl<T> HotaruWrite for T
where
    T: AsyncWrite + Unpin + Send + 'static,
{
    type Error = std::io::Error;
    type Buffered = tokio::io::BufWriter<Self>;

    fn into_buf_write(self) -> Self::Buffered {
        tokio::io::BufWriter::new(self)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        AsyncWriteExt::write(self, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::flush(self).await
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::shutdown(self).await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        AsyncWriteExt::write_all(self, buf).await
    }
}

impl<T> HotaruBufRead for T
where
    T: AsyncBufRead + AsyncRead + Unpin + Send + 'static,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        AsyncBufReadExt::fill_buf(self).await
    }

    fn consume(&mut self, amt: usize) {
        AsyncBufRead::consume(Pin::new(self), amt)
    }
}

impl<T> HotaruBufWrite for tokio::io::BufWriter<T>
where
    T: AsyncWrite + Unpin + Send + 'static,
{
}
