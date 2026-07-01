use core::future::Future;

use super::super::MaybeSend;

/// Async byte writer.
pub trait HotaruWrite {
    type Error: core::error::Error + Send + Sync + 'static;

    type Buffered: HotaruBufWrite<Error = Self::Error> + Unpin + MaybeSend + 'static;

    /// Consumes this writer and returns its buffered form.
    fn into_buf_write(self) -> Self::Buffered
    where
        Self: Sized;

    fn write<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a;

    fn flush(&mut self) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + '_;

    /// Default no-op for backends that rely on drop semantics
    /// (embedded-io-async); tokio blanket overrides this to send TCP FIN.
    fn shutdown(&mut self) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + '_ {
        async { Ok(()) }
    }

    /// Writes the entire buffer, looping until all bytes are consumed.
    /// Implementors signal "writer accepted 0 bytes" through their own
    /// `Self::Error`. Required for the same reason as `read_exact`.
    fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend;
}

pub trait HotaruBufWrite: HotaruWrite {}
