//! Tokio IO and TCP backend for Hotaru.

use core::pin::Pin;

use hotaru_core::connection::{HotaruBufRead, HotaruBufWrite, HotaruRead, HotaruWrite};
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
};

pub mod tcp;

pub use tcp::{
    TcpAccepter, TcpConnector, TcpConnectorAddr, TcpInbound, TcpMeta, TcpOutbound, TcpStream,
    TcpTransport,
};

/// Backend tag for Tokio IO values.
pub enum TokioBackend {}

/// Local Tokio IO adapter owned by `hotaru_io_tokio`.
///
/// This must be a real local newtype rather than `hotaru_core::IoCompat<T, _>`:
/// external crates cannot implement Hotaru's core traits for core-owned
/// generic aliases without violating Rust's orphan rules.
pub struct TokioIo<T> {
    inner: T,
}

impl<T> TokioIo<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> From<T> for TokioIo<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T> HotaruRead for TokioIo<T>
where
    T: AsyncRead + Unpin + Send + 'static,
{
    type Error = std::io::Error;
    type Buffered = TokioIo<tokio::io::BufReader<T>>;

    fn into_buf(self) -> Self::Buffered {
        TokioIo::new(tokio::io::BufReader::new(self.inner))
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

impl<T> HotaruWrite for TokioIo<T>
where
    T: AsyncWrite + Unpin + Send + 'static,
{
    type Error = std::io::Error;
    type Buffered = TokioIo<tokio::io::BufWriter<T>>;

    fn into_buf_write(self) -> Self::Buffered {
        TokioIo::new(tokio::io::BufWriter::new(self.inner))
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        AsyncWriteExt::write(&mut self.inner, buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::flush(&mut self.inner).await
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        AsyncWriteExt::shutdown(&mut self.inner).await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        AsyncWriteExt::write_all(&mut self.inner, buf).await
    }
}

impl<T> HotaruBufRead for TokioIo<T>
where
    T: AsyncBufRead + AsyncRead + Unpin + Send + 'static,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        AsyncBufReadExt::fill_buf(&mut self.inner).await
    }

    fn consume(&mut self, amt: usize) {
        AsyncBufRead::consume(Pin::new(&mut self.inner), amt)
    }
}

impl<T> HotaruBufWrite for TokioIo<T> where T: AsyncWrite + Unpin + Send + 'static {}
