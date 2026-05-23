//! HTTP/1 channel — shared I/O handles for the Protocol trait.
//!
//! `Http1Channel` wraps the split reader/writer halves of a `ConnStream`
//! and exposes real I/O methods (`parse_request`, `send_response`, etc.)
//! used by the protocol-level `handle` implementation.
//!
//! Connection metadata (local/remote addresses, ALPN, peer cert, etc.) is
//! stored as `Arc<W::Meta>` so the channel can be cheaply cloned without
//! requiring `ConnMeta: Clone`. Addresses are forwarded through the meta.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use hotaru_core::connection::{ConnMeta, ConnStream};
use hotaru_core::protocol::Channel;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use crate::channel::http_channel::HttpChannel;
use crate::message::body::HttpBody;
use crate::message::http_value::StatusCode;
use crate::message::request::HttpRequest;
use crate::message::response::HttpResponse;
use crate::protocol::error::HttpError;
use crate::security::safety::HttpSafety;

/// HTTP/1 channel for the Protocol trait.
///
/// Exposes shared I/O handles that can be used by the protocol-level
/// `handle` implementation. All I/O logic lives in the channel methods.
///
/// `meta` is the connection metadata captured at `ConnStream::split` time;
/// it is the canonical source for `local_addr` / `remote_addr` and any
/// transport-specific extension data (ALPN, peer cert, proxy headers).
pub struct Http1Channel<W: ConnStream> {
    reader: Arc<Mutex<BufReader<W::ReadHalf>>>,
    writer: Arc<Mutex<W::WriteHalf>>,
    meta: Arc<W::Meta>,
    open: Arc<AtomicBool>,
}

impl<W: ConnStream> Clone for Http1Channel<W> {
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
            writer: self.writer.clone(),
            meta: self.meta.clone(),
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

#[async_trait]
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
        if let Err(err) = request.send(&mut *writer).await {
            self.open.store(false, Ordering::Release);
            return Err(HttpError::Io(err));
        }
        if let Err(err) = writer.flush().await {
            self.open.store(false, Ordering::Release);
            return Err(HttpError::Io(err));
        }
        Ok(())
    }

    async fn parse_response(&self, safety: &HttpSafety) -> Result<HttpResponse, HttpError> {
        let mut reader = self.reader.lock().await;
        let response = HttpResponse::parse_lazy(&mut *reader, safety, false).await;

        // The current parser returns `HttpResponse::default()` on parse failure.
        // Treat an empty default response as a closed/broken channel for now.
        if response.meta.start_line.status_code() == StatusCode::OK
            && response.meta.header.is_empty()
            && matches!(response.body, HttpBody::Unparsed)
        {
            self.open.store(false, Ordering::Release);
            return Err(HttpError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "server closed connection",
            )));
        }

        Ok(response)
    }

    fn local_addr(&self) -> Option<SocketAddr> {
        self.meta.local_addr()
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        self.meta.remote_addr()
    }
}

impl<W: ConnStream> Http1Channel<W> {
    /// Creates a new HTTP/1 channel from a split wire and its meta.
    pub fn new(
        reader: BufReader<W::ReadHalf>,
        writer: W::WriteHalf,
        meta: W::Meta,
    ) -> Self {
        Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            meta: Arc::new(meta),
            open: Arc::new(AtomicBool::new(true)),
        }
    }
}
