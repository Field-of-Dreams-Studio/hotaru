//! embedded-io-async adapter backend for Hotaru.

#[cfg(feature = "spawn_local")]
use hotaru_core::connection::{
    HotaruBufRead, HotaruBufReader, HotaruBufWriter, HotaruIOError, HotaruRead, HotaruWrite,
    MaybeSend,
};

/// Backend tag for `embedded-io-async` IO values.
pub enum EmbeddedBackend {}

/// Local adapter for `embedded-io-async` IO sources.
///
/// Embedded IO values opt into Hotaru IO by wrapping in this newtype. The impl
/// target is local, so it cannot overlap with other backend adapters.
pub struct EmbeddedIo<T> {
    inner: T,
}

impl<T> EmbeddedIo<T> {
    /// Wraps an `embedded-io-async` value for use with Hotaru IO traits.
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

impl<T> From<T> for EmbeddedIo<T> {
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

#[cfg(feature = "spawn_local")]
impl<T> HotaruRead for EmbeddedIo<T>
where
    T: embedded_io_async::Read + Unpin + MaybeSend + 'static,
    T::Error: Into<HotaruIOError>,
{
    type Error = HotaruIOError;
    type Buffered = HotaruBufReader<Self>;

    fn into_buf(self) -> Self::Buffered {
        HotaruBufReader::new(self)
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Read::read(&mut self.inner, buf)
            .await
            .map_err(Into::into)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Read::read(&mut self.inner, &mut buf[total..])
                .await
                .map_err(Into::into)?
            {
                0 => return Err(HotaruIOError::UnexpectedEof),
                n => total += n,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "spawn_local")]
impl<T> HotaruWrite for EmbeddedIo<T>
where
    T: embedded_io_async::Write + Unpin + MaybeSend + 'static,
    T::Error: Into<HotaruIOError>,
{
    type Error = HotaruIOError;
    type Buffered = HotaruBufWriter<Self>;

    fn into_buf_write(self) -> Self::Buffered {
        HotaruBufWriter::new(self)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Write::write(&mut self.inner, buf)
            .await
            .map_err(Into::into)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io_async::Write::flush(&mut self.inner)
            .await
            .map_err(Into::into)
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Write::write(&mut self.inner, &buf[total..])
                .await
                .map_err(Into::into)?
            {
                0 => return Err(HotaruIOError::WriteZero),
                n => total += n,
            }
        }
        Ok(())
    }
}

#[cfg(feature = "spawn_local")]
impl<T> HotaruBufRead for EmbeddedIo<T>
where
    T: embedded_io_async::BufRead + embedded_io_async::Read + Unpin + MaybeSend + 'static,
    T::Error: Into<HotaruIOError>,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        embedded_io_async::BufRead::fill_buf(&mut self.inner)
            .await
            .map_err(Into::into)
    }

    fn consume(&mut self, amt: usize) {
        embedded_io_async::BufRead::consume(&mut self.inner, amt)
    }
}
