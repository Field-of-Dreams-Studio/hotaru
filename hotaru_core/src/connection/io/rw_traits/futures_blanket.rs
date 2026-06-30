use super::super::buf_reader::HotaruBufReader;
use super::super::buf_writer::HotaruBufWriter;
use super::super::IoCompat;
use super::{HotaruBufRead, HotaruBufWrite, HotaruRead, HotaruWrite};
use futures_util::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

/// Backend tag for `futures-io` IO types.
pub enum FuturesBackend {}

/// Backend-tagged adapter for `futures-io` IO sources.
///
/// `futures-io` values opt into the Hotaru IO traits by wrapping in
/// `FuturesIo` so the impls below target a distinct self-type instead of a
/// broad blanket over `T` (which would overlap with the Tokio blanket).
pub type FuturesIo<T> = IoCompat<T, FuturesBackend>;

impl<T> HotaruRead for FuturesIo<T>
where
    T: futures_io::AsyncRead + Unpin + Send + 'static,
{
    type Error = std::io::Error;
    type Buffered = HotaruBufReader<Self>;

    fn into_buf(self) -> Self::Buffered {
        HotaruBufReader::new(self)
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        AsyncReadExt::read(self.inner_mut(), buf).await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        AsyncReadExt::read_exact(self.inner_mut(), buf)
            .await
            .map(|_| ())
    }
}

impl<T> HotaruWrite for FuturesIo<T>
where
    T: futures_io::AsyncWrite + Unpin + Send + 'static,
{
    type Error = std::io::Error;
    type Buffered = HotaruBufWriter<Self>;

    fn into_buf_write(self) -> Self::Buffered {
        HotaruBufWriter::new(self)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        AsyncWriteExt::write(self.inner_mut(), buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::flush(self.inner_mut()).await
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::close(self.inner_mut()).await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        AsyncWriteExt::write_all(self.inner_mut(), buf).await
    }
}

impl<T> HotaruBufRead for FuturesIo<T>
where
    T: futures_io::AsyncBufRead + futures_io::AsyncRead + Unpin + Send + 'static,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        AsyncBufReadExt::fill_buf(self.inner_mut()).await
    }

    fn consume(&mut self, amt: usize) {
        AsyncBufReadExt::consume_unpin(self.inner_mut(), amt)
    }
}

impl<T> HotaruBufWrite for FuturesIo<T> where T: futures_io::AsyncWrite + Unpin + Send + 'static {}
