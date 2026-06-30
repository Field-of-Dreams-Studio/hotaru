use super::super::buf_reader::HotaruBufReader;
use super::super::buf_writer::HotaruBufWriter;
use super::super::IoCompat;
use super::{HotaruBufRead, HotaruBufWrite, HotaruIOError, HotaruRead, HotaruWrite};

/// Backend tag for `embedded-io-async` IO types.
pub enum EmbeddedBackend {}

/// Backend-tagged adapter for `embedded-io-async` IO sources.
///
/// Embedded IO values opt into the Hotaru IO traits by wrapping in
/// `EmbeddedIo` so the impls below target a distinct self-type instead of a
/// broad blanket over `T`.
pub type EmbeddedIo<T> = IoCompat<T, EmbeddedBackend>;

impl<T> HotaruRead for EmbeddedIo<T>
where
    T: embedded_io_async::Read + Unpin + 'static,
    T::Error: Into<HotaruIOError>,
{
    type Error = HotaruIOError;
    type Buffered = HotaruBufReader<Self>;

    fn into_buf(self) -> Self::Buffered {
        HotaruBufReader::new(self)
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Read::read(self.inner_mut(), buf)
            .await
            .map_err(Into::into)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Read::read(self.inner_mut(), &mut buf[total..])
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

impl<T> HotaruWrite for EmbeddedIo<T>
where
    T: embedded_io_async::Write + Unpin + 'static,
    T::Error: Into<HotaruIOError>,
{
    type Error = HotaruIOError;
    type Buffered = HotaruBufWriter<Self>;

    fn into_buf_write(self) -> Self::Buffered {
        HotaruBufWriter::new(self)
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Write::write(self.inner_mut(), buf)
            .await
            .map_err(Into::into)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io_async::Write::flush(self.inner_mut())
            .await
            .map_err(Into::into)
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Write::write(self.inner_mut(), &buf[total..])
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

impl<T> HotaruBufRead for EmbeddedIo<T>
where
    T: embedded_io_async::BufRead + embedded_io_async::Read + Unpin + 'static,
    T::Error: Into<HotaruIOError>,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        embedded_io_async::BufRead::fill_buf(self.inner_mut())
            .await
            .map_err(Into::into)
    }

    fn consume(&mut self, amt: usize) {
        embedded_io_async::BufRead::consume(self.inner_mut(), amt)
    }
}
