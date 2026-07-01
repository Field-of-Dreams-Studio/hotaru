//! futures-io adapter backend for Hotaru.

use hotaru_core::connection::{
    HotaruBufRead, HotaruBufReader, HotaruBufWrite, HotaruBufWriter, HotaruRead, HotaruWrite,
};

use futures_util::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

/// Backend tag for `futures-io` IO values.
pub enum FuturesBackend {}

/// Local adapter for `futures-io` IO sources.
///
/// `futures-io` values opt into Hotaru IO by wrapping in this newtype. The
/// impl target is local, so it cannot overlap with other backend adapters.
pub struct FuturesIo<T> {
    inner: T,
}

impl<T> FuturesIo<T> {
    /// Wraps a `futures-io` value for use with Hotaru IO traits.
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

impl<T> From<T> for FuturesIo<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

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
        AsyncReadExt::read(&mut self.inner, buf).await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        AsyncReadExt::read_exact(&mut self.inner, buf)
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
        AsyncWriteExt::write(&mut self.inner, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::flush(&mut self.inner).await
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::close(&mut self.inner).await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        AsyncWriteExt::write_all(&mut self.inner, buf).await
    }
}

impl<T> HotaruBufRead for FuturesIo<T>
where
    T: futures_io::AsyncBufRead + futures_io::AsyncRead + Unpin + Send + 'static,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        AsyncBufReadExt::fill_buf(&mut self.inner).await
    }

    fn consume(&mut self, amt: usize) {
        AsyncBufReadExt::consume_unpin(&mut self.inner, amt)
    }
}

impl<T> HotaruBufWrite for FuturesIo<T> where T: futures_io::AsyncWrite + Unpin + Send + 'static {}
