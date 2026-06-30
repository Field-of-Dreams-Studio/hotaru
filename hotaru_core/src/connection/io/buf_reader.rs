use super::{HotaruBufRead, HotaruRead, MaybeSend};
use alloc::vec::Vec;

/// Concrete buffered reader generic over any `HotaruRead`.
///
/// Works across both std (tokio) and embedded (embedded-io-async) backends
/// because the `HotaruRead` abstraction is feature-agnostic. The API
/// mirrors `tokio::io::BufReader<R>` for familiarity.
pub struct HotaruBufReader<R> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
}

impl<R> HotaruBufReader<R> {
    pub const DEFAULT_CAPACITY: usize = 8 * 1024;

    pub fn new(inner: R) -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY, inner)
    }

    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        let mut buf = Vec::with_capacity(capacity);
        buf.resize(capacity, 0);
        Self {
            inner,
            buf,
            pos: 0,
            cap: 0,
        }
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
    pub fn get_ref(&self) -> &R {
        &self.inner
    }
    /// Currently-buffered unconsumed bytes.
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }
}

impl<R> From<R> for HotaruBufReader<R>
where
    R: HotaruRead + MaybeSend,
{
    fn from(inner: R) -> Self {
        Self::new(inner)
    }
}

impl<R: HotaruRead + MaybeSend + Unpin + 'static> HotaruRead for HotaruBufReader<R> {
    type Error = R::Error;
    type Buffered = Self;

    fn into_buf(self) -> Self::Buffered {
        self
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // Drain the internal buffer first.
        if self.pos < self.cap {
            let n = (self.cap - self.pos).min(buf.len());
            buf[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
            self.pos += n;
            return Ok(n);
        }
        // Internal buffer is empty.
        if buf.len() >= self.buf.len() {
            // Large request — bypass internal buffer.
            return self.inner.read(buf).await;
        }
        // Refill internal buffer, then copy.
        self.cap = self.inner.read(&mut self.buf).await?;
        self.pos = 0;
        let n = self.cap.min(buf.len());
        buf[..n].copy_from_slice(&self.buf[..n]);
        self.pos += n;
        Ok(n)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        // Serve as many bytes as possible from the internal buffer, then defer
        // the remainder (including the EOF sentinel) to the inner reader's own
        // `read_exact` — `Self::Error = R::Error`, so the inner backend owns
        // the `UnexpectedEof` construction.
        let mut filled = 0;
        if self.pos < self.cap {
            let n = (self.cap - self.pos).min(buf.len());
            buf[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
            self.pos += n;
            filled = n;
        }
        if filled < buf.len() {
            self.inner.read_exact(&mut buf[filled..]).await?;
        }
        Ok(())
    }
}

impl<R: HotaruRead + MaybeSend + Unpin + 'static> HotaruBufRead for HotaruBufReader<R> {
    async fn fill_buf<'a>(&'a mut self) -> Result<&'a [u8], Self::Error> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf).await?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = (self.pos + amt).min(self.cap);
    }
}
