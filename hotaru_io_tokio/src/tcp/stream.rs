//! TCP wire stream and metadata.

use core::net::SocketAddr;

use hotaru_core::connection::{ConnMeta, ConnStream, HotaruRead, HotaruWrite};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream as TokioTcpStream,
};

use crate::TokioIo;

/// Tokio TCP stream wrapper owned by `hotaru_io_tokio`.
pub struct TcpStream {
    inner: TokioTcpStream,
}

impl TcpStream {
    pub fn new(inner: TokioTcpStream) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> TokioTcpStream {
        self.inner
    }

    pub fn inner(&self) -> &TokioTcpStream {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut TokioTcpStream {
        &mut self.inner
    }

    pub async fn connect<T: tokio::net::ToSocketAddrs>(target: T) -> std::io::Result<Self> {
        TokioTcpStream::connect(target).await.map(Self::new)
    }
}

impl HotaruRead for TcpStream {
    type Error = std::io::Error;
    type Buffered = <TokioIo<TokioTcpStream> as HotaruRead>::Buffered;

    fn into_buf(self) -> Self::Buffered {
        TokioIo::new(self.inner).into_buf()
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

impl HotaruWrite for TcpStream {
    type Error = std::io::Error;
    type Buffered = <TokioIo<TokioTcpStream> as HotaruWrite>::Buffered;

    fn into_buf_write(self) -> Self::Buffered {
        TokioIo::new(self.inner).into_buf_write()
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

/// Connection metadata for plain TCP.
pub struct TcpMeta {
    local: Option<SocketAddr>,
    remote: Option<SocketAddr>,
}

impl ConnMeta for TcpMeta {
    fn local_addr(&self) -> Option<SocketAddr> {
        self.local
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote
    }
}

impl ConnStream for TcpStream {
    type ReadHalf = TokioIo<tokio::io::ReadHalf<TokioTcpStream>>;
    type WriteHalf = TokioIo<tokio::io::WriteHalf<TokioTcpStream>>;
    type Meta = TcpMeta;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta) {
        let meta = TcpMeta {
            local: self.inner.local_addr().ok(),
            remote: self.inner.peer_addr().ok(),
        };
        let (read, write) = tokio::io::split(self.inner);
        (TokioIo::new(read), TokioIo::new(write), meta)
    }

    fn peer_addr(&self) -> Option<SocketAddr> {
        self.inner.peer_addr().ok()
    }

    fn local_addr(&self) -> Option<SocketAddr> {
        self.inner.local_addr().ok()
    }
}
