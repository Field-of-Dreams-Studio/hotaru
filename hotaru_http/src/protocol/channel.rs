//! HTTP/1 channel — shared I/O handles for the Protocol trait.
//!
//! `Http1Channel` wraps the split reader/writer halves of a `ConnStream`
//! and exposes real I/O methods (`parse_request`, `send_response`, etc.)
//! used by the protocol-level `handle` implementation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use hotaru_core::connection::ConnStream;
use hotaru_core::protocol::Channel;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use crate::message::request::HttpRequest;
use crate::message::response::HttpResponse;
use crate::protocol::error::HttpError;
use crate::protocol::http_channel::HttpChannel;
use crate::protocol::transport::HttpTransport;
use crate::security::safety::HttpSafety;

/// HTTP/1 channel for the Protocol trait.
///
/// Exposes shared I/O handles that can be used by the protocol-level
/// `handle` implementation. All I/O logic lives in the channel methods.
pub struct Http1Channel<W: ConnStream> {
    reader: Arc<Mutex<BufReader<W::ReadHalf>>>,
    writer: Arc<Mutex<W::WriteHalf>>,
    transport: Arc<Mutex<HttpTransport>>,
    open: Arc<AtomicBool>,
}

impl<W: ConnStream> Clone for Http1Channel<W> {
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
            writer: self.writer.clone(),
            transport: self.transport.clone(),
            open: self.open.clone(),
        }
    }
}

impl<W: ConnStream> Channel for Http1Channel<W> {
    fn is_open(&self) -> bool {
        self.open.load(Ordering::Acquire)
    }

    fn close(&self) {
        self.open.store(false, Ordering::Release);
    }
}

impl<W: ConnStream> HttpChannel for Http1Channel<W> {
    async fn parse_request(&self, safety: &HttpSafety) -> Result<HttpRequest, HttpError> {
        let mut reader = self.reader.lock().await;
        let request = HttpRequest::parse_lazy(&mut *reader, safety, false).await;

        // EOF / malformed: flip the channel closed and signal Io.
        if request.meta.path().is_empty() && request.meta.header.is_empty() {
            self.open.store(false, Ordering::Release);
            return Err(HttpError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "client closed connection",
            )));
        }
        Ok(request)
    }

    async fn send_response(&self, response: HttpResponse) -> Result<(), HttpError> {
        let mut writer = self.writer.lock().await;
        response.send(&mut *writer).await.map_err(HttpError::Io)?;
        writer.flush().await.map_err(HttpError::Io)?;
        Ok(())
    }

    async fn send_request(&self, request: HttpRequest) -> Result<(), HttpError> {
        let mut writer = self.writer.lock().await;
        request.send(&mut *writer).await.map_err(HttpError::Io)?;
        writer.flush().await.map_err(HttpError::Io)?;
        Ok(())
    }

    async fn parse_response(&self, safety: &HttpSafety) -> Result<HttpResponse, HttpError> {
        let mut reader = self.reader.lock().await;
        Ok(HttpResponse::parse_lazy(&mut *reader, safety, false).await)
    }
}

impl<W: ConnStream> Http1Channel<W> {
    /// Creates a new HTTP/1 channel.
    pub fn new(
        reader: BufReader<W::ReadHalf>,
        writer: W::WriteHalf,
        transport: HttpTransport,
    ) -> Self {
        Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            transport: Arc::new(Mutex::new(transport)),
            open: Arc::new(AtomicBool::new(true)),
        }
    }
}
