use core::future::Future;

use super::super::MaybeSend;

/// Async byte reader.
pub trait HotaruRead {
    /// Concrete error returned by this reader. A backend picks `HotaruIOError`
    /// or its own type (e.g. tokio uses `std::io::Error`).
    type Error: core::error::Error + Send + Sync + 'static;

    type Buffered: HotaruBufRead<Error = Self::Error> + Unpin + MaybeSend + 'static;

    /// Consumes this reader and returns its buffered form.  
    fn into_buf(self) -> Self::Buffered
    where
        Self: Sized;

    /// Reads bytes into `buf`, returning the number written.
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a;

    /// Reads exactly `buf.len()` bytes. Implementors signal "EOF before the
    /// buffer was filled" through their own `Self::Error` (concrete impls use
    /// `HotaruIOError::UnexpectedEof`; tokio uses `ErrorKind::UnexpectedEof`).
    /// Required — the trait definition stays in terms of `Self::Error` only,
    /// so the sentinel construction lives in the concrete impl.
    fn read_exact<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend;
}

/// Buffered async byte reader. Carries protocol-detection peeked bytes
/// through `Protocol::open_channel` without leaking `tokio::io::BufReader`.
pub trait HotaruBufRead: HotaruRead {
    /// Returns a slice of the currently buffered bytes, filling the buffer
    /// from the underlying reader if it's empty.
    fn fill_buf<'a>(
        &'a mut self,
    ) -> impl Future<Output = Result<&'a [u8], Self::Error>> + MaybeSend + 'a;

    /// Marks the first `amt` bytes of the internal buffer as consumed so
    /// the next `fill_buf` skips them.
    fn consume(&mut self, amt: usize);

    /// Reads bytes into `buf` until the delimiter `byte` is encountered,
    /// inclusive. Stops at EOF without error. Returns `Self::Error` directly:
    /// EOF is `Ok`, and the only failure path is `fill_buf`, which already
    /// yields `Self::Error` (propagated via `?`). No sentinel is needed here,
    /// so this stays a default method that never names `HotaruIOError`.
    fn read_until<'a>(
        &'a mut self,
        byte: u8,
        buf: &'a mut alloc::vec::Vec<u8>,
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            let mut read = 0;
            loop {
                let (done, used) = {
                    let available = self.fill_buf().await?;
                    if available.is_empty() {
                        return Ok(read);
                    }
                    if let Some(i) = available.iter().position(|b| *b == byte) {
                        buf.extend_from_slice(&available[..=i]);
                        (true, i + 1)
                    } else {
                        buf.extend_from_slice(available);
                        (false, available.len())
                    }
                };
                self.consume(used);
                read += used;
                if done {
                    return Ok(read);
                }
            }
        }
    }

    /// Reads a line into `buf` (up to and including the next `\n`). Also a
    /// default method returning `Self::Error`; builds on `read_until`.
    fn read_line<'a>(
        &'a mut self,
        buf: &'a mut alloc::string::String,
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend + 'a
    where
        Self: MaybeSend,
    {
        async move {
            let mut bytes = alloc::vec::Vec::new();
            let n = self.read_until(b'\n', &mut bytes).await?;
            buf.push_str(&alloc::string::String::from_utf8_lossy(&bytes));
            Ok(n)
        }
    }
}
