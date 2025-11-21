//! IO compatibility layer between Hotaru's TcpConnectionStream and Hyper's requirements

use std::pin::Pin;
use std::task::{Context, Poll};
use std::io;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf, BufReader, BufWriter, ReadHalf, WriteHalf};
use hotaru_core::connection::TcpConnectionStream;
use hyper::rt::{Read, Write};

/// Wrapper to make TcpConnectionStream compatible with Hyper's IO traits
pub struct HyperIoCompat {
    inner: TcpConnectionStream,
}

/// Hyper compatibility wrapper for buffered streams that directly uses buffered readers
pub struct BufferedHyperIoCompat {
    reader: BufReader<ReadHalf<TcpConnectionStream>>,
    writer: BufWriter<WriteHalf<TcpConnectionStream>>,
}

impl HyperIoCompat {
    pub fn new(stream: TcpConnectionStream) -> Self {
        Self { inner: stream }
    }
    
    pub fn new_buffered(reader: BufReader<ReadHalf<TcpConnectionStream>>, writer: BufWriter<WriteHalf<TcpConnectionStream>>) -> BufferedHyperIoCompat {
        BufferedHyperIoCompat { reader, writer }
    }
}


// Implement tokio's AsyncRead
impl AsyncRead for HyperIoCompat {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

// Implement tokio's AsyncWrite
impl AsyncWrite for HyperIoCompat {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// Implement hyper's Read trait
impl Read for HyperIoCompat {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        // Create a tokio ReadBuf from the hyper ReadBufCursor
        let mut read_buf = unsafe {
            ReadBuf::uninit(buf.as_mut())
        };
        
        // Use our AsyncRead implementation
        match Pin::new(&mut self.inner).poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let filled = read_buf.filled().len();
                unsafe { buf.advance(filled) };
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

// Implement hyper's Write trait
impl Write for HyperIoCompat {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// Implement AsyncRead for BufferedHyperIoCompat
impl AsyncRead for BufferedHyperIoCompat {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

// Implement AsyncWrite for BufferedHyperIoCompat
impl AsyncWrite for BufferedHyperIoCompat {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.writer).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.writer).poll_shutdown(cx)
    }
}

// Implement hyper's Read trait for BufferedHyperIoCompat
impl Read for BufferedHyperIoCompat {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        // Create a tokio ReadBuf from the hyper ReadBufCursor
        let mut read_buf = unsafe {
            ReadBuf::uninit(buf.as_mut())
        };
        
        // Use our buffered reader
        match Pin::new(&mut self.reader).poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let filled = read_buf.filled().len();
                unsafe { buf.advance(filled) };
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

// Implement hyper's Write trait for BufferedHyperIoCompat
impl Write for BufferedHyperIoCompat {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.writer).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.writer).poll_shutdown(cx)
    }
}