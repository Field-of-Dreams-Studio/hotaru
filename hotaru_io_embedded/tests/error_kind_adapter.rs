#![cfg(feature = "spawn_local")]

use hotaru_core::connection::{HotaruBufRead, HotaruRead, HotaruWrite};
use hotaru_io_embedded::{EmbeddedIo, EmbeddedIoError};

struct ErrorKindIo {
    bytes: [u8; 1],
}

impl embedded_io_async::ErrorType for ErrorKindIo {
    type Error = embedded_io_async::ErrorKind;
}

impl embedded_io_async::Read for ErrorKindIo {
    async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Err(embedded_io_async::ErrorKind::Interrupted)
    }
}

impl embedded_io_async::BufRead for ErrorKindIo {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Ok(&self.bytes)
    }

    fn consume(&mut self, _amt: usize) {}
}

impl embedded_io_async::Write for ErrorKindIo {
    async fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        Err(embedded_io_async::ErrorKind::TimedOut)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn assert_hotaru_io<T>()
where
    T: HotaruRead<Error = EmbeddedIoError>
        + HotaruBufRead<Error = EmbeddedIoError>
        + HotaruWrite<Error = EmbeddedIoError>,
{
}

#[test]
fn embedded_error_kind_io_gets_hotaru_traits() {
    assert_hotaru_io::<EmbeddedIo<ErrorKindIo>>();
}

#[test]
fn embedded_io_error_preserves_backend_kind() {
    let error = EmbeddedIoError::Backend(embedded_io_async::ErrorKind::TimedOut);
    assert_eq!(error.kind(), embedded_io_async::ErrorKind::TimedOut);
    assert_eq!(
        EmbeddedIoError::WriteZero.kind(),
        embedded_io_async::ErrorKind::WriteZero
    );
}
