//! TlsStream — unified wire type for TransportSpec.

use rustls::pki_types::CertificateDer;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream as ClientTlsStream;
use tokio_rustls::server::TlsStream as ServerTlsStream;

use hotaru_core::connection::{ConnMeta, ConnStream, HotaruRead, HotaruWrite};
use hotaru_io_tokio::TokioIo;

/// Connection metadata captured at split-time for TLS streams.
pub struct TlsMeta {
    local: Option<SocketAddr>,
    remote: Option<SocketAddr>,
    /// Peer certificate chain captured at TLS handshake completion.
    /// `None` for client-side streams and for server-side streams
    /// configured with `ClientAuth::None`. Leaf cert is first.
    peer_certificates: Option<Arc<[CertificateDer<'static>]>>,
}

impl TlsMeta {
    pub fn new(local: Option<SocketAddr>, remote: Option<SocketAddr>) -> Self {
        Self {
            local,
            remote,
            peer_certificates: None,
        }
    }

    pub fn peer_certificates(&self) -> Option<&[CertificateDer<'static>]> {
        self.peer_certificates.as_deref()
    }
}

impl ConnMeta for TlsMeta {
    fn local_addr(&self) -> Option<SocketAddr> {
        self.local
    }
    fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote
    }
}

// ============================================================================
// TlsStream — unified enum required by TransportSpec
// ============================================================================

/// Unified TLS stream wrapping both client-side and server-side variants.
///
/// `TransportSpec` requires `Accepter::Stream == Connector::Stream == Wire`.
/// Because `tokio_rustls` gives distinct `client::TlsStream` and `server::TlsStream`
/// types, this enum is the bridge.
pub enum TlsStream {
    Client(ClientTlsStream<TcpStream>),
    Server(ServerTlsStream<TcpStream>),
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TlsStream::Client(s) => Pin::new(s).poll_read(cx, buf),
            TlsStream::Server(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            TlsStream::Client(s) => Pin::new(s).poll_write(cx, buf),
            TlsStream::Server(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TlsStream::Client(s) => Pin::new(s).poll_flush(cx),
            TlsStream::Server(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TlsStream::Client(s) => Pin::new(s).poll_shutdown(cx),
            TlsStream::Server(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl HotaruRead for TlsStream {
    type Error = std::io::Error;
    type Buffered = TokioIo<tokio::io::BufReader<Self>>;

    fn into_buf(self) -> Self::Buffered {
        TokioIo::new(tokio::io::BufReader::new(self))
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        AsyncReadExt::read(self, buf).await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        AsyncReadExt::read_exact(self, buf).await.map(|_| ())
    }
}

impl HotaruWrite for TlsStream {
    type Error = std::io::Error;
    type Buffered = TokioIo<tokio::io::BufWriter<Self>>;

    fn into_buf_write(self) -> Self::Buffered {
        TokioIo::new(tokio::io::BufWriter::new(self))
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

impl ConnStream for TlsStream {
    type ReadHalf = TokioIo<tokio::io::ReadHalf<TlsStream>>;
    type WriteHalf = TokioIo<tokio::io::WriteHalf<TlsStream>>;
    type Meta = TlsMeta;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta) {
        let peer_certificates = match &self {
            TlsStream::Client(_) => None,
            TlsStream::Server(s) => s
                .get_ref()
                .1
                .peer_certificates()
                .map(|certs| Arc::from(certs.to_vec().into_boxed_slice())),
        };
        let meta = TlsMeta {
            local: self.local_addr(),
            remote: self.peer_addr(),
            peer_certificates,
        };
        let (r, w) = tokio::io::split(self);
        (TokioIo::new(r), TokioIo::new(w), meta)
    }

    fn peer_addr(&self) -> Option<SocketAddr> {
        match self {
            TlsStream::Client(s) => s.get_ref().0.peer_addr(),
            TlsStream::Server(s) => s.get_ref().0.peer_addr(),
        }
        .ok()
    }

    fn local_addr(&self) -> Option<SocketAddr> {
        match self {
            TlsStream::Client(s) => s.get_ref().0.local_addr(),
            TlsStream::Server(s) => s.get_ref().0.local_addr(),
        }
        .ok()
    }
}
