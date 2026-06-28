use super::super::buf_reader::HotaruBufReader;
use super::super::buf_writer::HotaruBufWriter;
use super::{HotaruBufRead, HotaruBufWrite, HotaruIOError, HotaruRead, HotaruWrite};

impl<T> HotaruRead for T
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
        embedded_io_async::Read::read(self, buf)
            .await
            .map_err(Into::into)
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Read::read(self, &mut buf[total..])
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

impl<T> HotaruWrite for T
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
        embedded_io_async::Write::write(self, buf)
            .await
            .map_err(Into::into)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        embedded_io_async::Write::flush(self)
            .await
            .map_err(Into::into)
    }

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let mut total = 0;
        while total < buf.len() {
            match embedded_io_async::Write::write(self, &buf[total..])
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

impl<T> HotaruBufRead for T
where
    T: embedded_io_async::BufRead + embedded_io_async::Read + Unpin + 'static,
    T::Error: Into<HotaruIOError>,
{
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        embedded_io_async::BufRead::fill_buf(self)
            .await
            .map_err(Into::into)
    }

    fn consume(&mut self, amt: usize) {
        embedded_io_async::BufRead::consume(self, amt)
    }
}
