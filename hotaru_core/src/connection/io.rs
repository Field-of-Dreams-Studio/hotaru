//! Framework-owned async IO traits.

pub mod marker; 
pub use marker::{IoCompat, MaybeSend};

pub mod rw_traits; 
pub use rw_traits::{HotaruBufRead, HotaruIOError, HotaruRead, HotaruWrite, HotaruBufWrite}; 

#[cfg(feature = "io_tokio")]
pub use rw_traits::{TokioBackend, TokioIo};
#[cfg(feature = "io_embedded")]
pub use rw_traits::{EmbeddedBackend, EmbeddedIo};
#[cfg(feature = "io_futures")]
pub use rw_traits::{FuturesBackend, FuturesIo};

pub mod buf_reader; 
pub mod buf_writer; 

pub use buf_reader::HotaruBufReader;
pub use buf_writer::HotaruBufWriter;

/// The buffered read-half type selected by a transport's wire stream.
///
/// Resolves to whatever `<ReadHalf as HotaruRead>::Buffered` the active
/// backend chose (tokio -> `tokio::io::BufReader`, embedded/fallback ->
/// `HotaruBufReader`). Stage 7 call sites use this instead of hardcoding a
/// concrete buffered reader.
pub type BufferedReadHalf<TS> =
    <<<TS as crate::connection::TransportSpec>::Wire as crate::connection::ConnStream>::ReadHalf
        as HotaruRead>::Buffered;

/// The buffered write-half type selected by a transport's wire stream.
///
/// Resolves to whatever `<WriteHalf as HotaruWrite>::Buffered` the active
/// backend chose (tokio -> `tokio::io::BufWriter`, embedded/fallback ->
/// `HotaruBufWriter`).
pub type BufferedWriteHalf<TS> =
    <<<TS as crate::connection::TransportSpec>::Wire as crate::connection::ConnStream>::WriteHalf
        as HotaruWrite>::Buffered;
