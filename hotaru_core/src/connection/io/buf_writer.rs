use super::{HotaruWrite, HotaruBufWrite, MaybeSend};

pub struct HotaruBufWriter<W> {
    inner: W,
}

impl<W> HotaruBufWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}

impl<W> From<W> for HotaruBufWriter<W>
where
    W: HotaruWrite + MaybeSend,
{
    fn from(inner: W) -> Self {
        Self::new(inner)
    }
}

impl<W> HotaruWrite for HotaruBufWriter<W>
where
    W: HotaruWrite + MaybeSend + Unpin + 'static,
{
    type Error = W::Error;
    type Buffered = Self;

    fn into_buf_write(self) -> Self::Buffered {
        self
    }

    // Basic first impl: forward write/flush/shutdown/write_all to `inner`;
    // add real buffering internals later only when a call site needs them.
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.inner.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.inner.flush().await
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.inner.shutdown().await
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.inner.write_all(buf).await
    }
}

impl<W> HotaruBufWrite for HotaruBufWriter<W>
where
    W: HotaruWrite + MaybeSend + Unpin + 'static,
{} 
