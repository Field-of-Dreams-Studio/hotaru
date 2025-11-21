//! This module provides an abstraction over plain TCP and TLS connections using Tokio.
//!
//! The `TcpConnectionStream` enum encapsulates either a plain TCP stream or a TLS stream.
//! This allows consumers to work with either connection type transparently.
//!
//! By separating the connection from buffering, users of this module can choose to apply buffering
//! (e.g., via `tokio::io::BufReader` or `tokio::io::BufWriter`) as necessary in their application.

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf, ReadHalf, WriteHalf}; 
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

/// Represents a connection which can be either plain TCP or secured with TLS.
pub enum TcpConnectionStream {
    /// A plain TCP connection.
    Tcp(TcpStream),
    /// A secure TLS connection built on top of a TCP stream.
    Tls(TlsStream<TcpStream>),
}

impl TcpConnectionStream {
    /// Reconstructs a TcpConnectionStream from split read and write halves.
    /// 
    /// This is used after protocol detection to pass the full stream to the protocol handler.
    pub fn from_parts(read_half: ReadHalf<Self>, write_half: WriteHalf<Self>) -> Self {
        read_half.unsplit(write_half)
    }
    
    /// Creates a new `Connection` instance wrapping a plain TCP stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TcpStream` representing the underlying TCP connection.
    ///
    /// # Returns
    ///
    /// A `Connection::Tcp` variant wrapping the provided `TcpStream`.
    pub fn new_tcp(stream: TcpStream) -> Self {
        TcpConnectionStream::Tcp(stream)
    }

    /// Creates a new `Connection` instance wrapping a TLS stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - A `TlsStream<TcpStream>` representing the underlying TLS-secured connection.
    ///
    /// # Returns
    ///
    /// A `Connection::Tls` variant wrapping the provided `TlsStream<TcpStream>`.
    pub fn new_tls(stream: TlsStream<TcpStream>) -> Self {
        TcpConnectionStream::Tls(stream)
    } 
    

    /// Provides mutable access to the underlying stream for read operations.
    ///
    /// # Returns
    ///
    /// A mutable reference to a type that implements `AsyncRead`. This can be used to perform
    /// read operations on the connection.
    pub fn reader_mut(&mut self) -> &mut (dyn AsyncRead + Unpin) {
        match self {
            TcpConnectionStream::Tcp(stream) => stream,
            TcpConnectionStream::Tls(stream) => stream,
        }
    } 

    /// Splits the connection into separate read and write halves.
    ///
    /// Note: This uses `tokio::io::split` to separate the underlying stream. The returned halves
    /// can be used concurrently in separate tasks.
    ///
    /// # Returns
    ///
    /// A tuple where the first element is the read half implementing `AsyncRead`
    /// and the second element is the write half implementing `AsyncWrite`.
    pub fn split(self) -> (ReadHalf<Self>, WriteHalf<Self>)
    where
        Self: AsyncRead + AsyncWrite + Unpin,
    {
        io::split(self)
    } 

    /// Provides mutable access to the underlying stream for write operations.
    ///
    /// # Returns
    ///
    /// A mutable reference to a type that implements `AsyncWrite`. This can be used to perform
    /// write operations on the connection.
    pub fn writer_mut(&mut self) -> &mut (dyn AsyncWrite + Unpin) {
        match self {
            TcpConnectionStream::Tcp(stream) => stream,
            TcpConnectionStream::Tls(stream) => stream,
        }
    } 

    /// Gracefully shuts down the connection by closing the write half.
    ///
    /// This sends a FIN packet (TCP) or TLS close_notify alert to notify the peer
    /// that no more data will be sent. Reads can still be performed after shutdown.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use hotaru_core::connection::stream::TcpConnectionStream;
    /// # async fn example(mut conn: TcpConnectionStream) {
    /// let _ = conn.shutdown().await;
    /// # }
    /// ```
    pub async fn shutdown(&mut self) -> std::io::Result<()> {
        // Use pattern matching to call the appropriate shutdown method
        match self {
            TcpConnectionStream::Tcp(stream) => stream.shutdown().await,
            TcpConnectionStream::Tls(stream) => stream.shutdown().await,
        }
    } 
}

impl AsyncRead for TcpConnectionStream {
    /// Polls the `Connection` for reading data asynchronously.
    ///
    /// This implementation delegates the read operation to the u nderlying stream, whether
    /// it is a plain TCP or TLS stream.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    /// * `buf` - The buffer for storing the read data.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating if the operation is ready or pending.
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>, 
    ) -> Poll<std::io::Result<()>> {
        // Convert the pinned reference of self into a mutable reference to the enum, then match on it.
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TcpConnectionStream {
    /// Polls the `Connection` for writing data asynchronously.
    ///
    /// This implementation delegates the write operation to the underlying stream, whether
    /// it is a plain TCP or TLS stream.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    /// * `buf` - The buffer containing data to write.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating the result of the write operation.
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    /// Polls the `Connection` for flushing written data asynchronously.
    ///
    /// This operation ensures that all buffered data in the underlying stream is pushed out.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating the result of the flush operation.
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_flush(cx),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    /// Polls the `Connection` to shutdown the write half of the connection asynchronously.
    ///
    /// This operation signals that no more data will be written to the connection.
    ///
    /// # Arguments
    ///
    /// * `cx` - The asynchronous task context.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating the result of the shutdown operation.
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            TcpConnectionStream::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
            TcpConnectionStream::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
} 
