//! Framework-owned async IO traits.

pub use crate::marker::{MaybeSend, MaybeSendBoxFuture};

pub mod rw_traits;
pub use rw_traits::{HotaruBufRead, HotaruBufWrite, HotaruIOError, HotaruRead, HotaruWrite};

pub mod buf_reader;
pub mod buf_writer;

pub use buf_reader::HotaruBufReader;
pub use buf_writer::HotaruBufWriter;

/// The buffered read-half type selected by a transport's wire stream.
///
/// Resolves to whatever `<ReadHalf as HotaruRead>::Buffered` the active
/// backend chose (external backend -> backend buffer, embedded/fallback ->
/// `HotaruBufReader`). Stage 7 call sites use this instead of hardcoding a
/// concrete buffered reader.
pub type BufferedReadHalf<TS> =
    <<<TS as crate::connection::TransportSpec>::Wire as crate::connection::ConnStream>::ReadHalf
        as HotaruRead>::Buffered;

/// The buffered write-half type selected by a transport's wire stream.
///
/// Resolves to whatever `<WriteHalf as HotaruWrite>::Buffered` the active
/// backend chose (external backend -> backend buffer, embedded/fallback ->
/// `HotaruBufWriter`).
pub type BufferedWriteHalf<TS> =
    <<<TS as crate::connection::TransportSpec>::Wire as crate::connection::ConnStream>::WriteHalf
        as HotaruWrite>::Buffered;
