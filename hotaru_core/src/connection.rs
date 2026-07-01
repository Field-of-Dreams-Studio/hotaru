/// Connection compatibility helpers.
pub mod connection;
/// Connection error and result types.
pub mod error;
/// Hotaru async read/write traits and buffered wrappers.
pub mod io;
/// Accepter and connector primitive traits.
pub mod primitive;
/// Inbound and outbound transport runtime traits.
pub mod runtime;
/// Transport-neutral stream and metadata traits.
pub mod stream;
/// Connection tests and test-only helpers.
pub mod test;
/// Test support utilities for connection implementations.
#[cfg(test)]
pub mod test_support;
/// Transport family specification trait.
pub mod transport_spec;

pub use self::error::Result;
pub use self::io::buf_reader::HotaruBufReader;
pub use self::io::buf_writer::HotaruBufWriter;
pub use self::io::{
    BufferedReadHalf, BufferedWriteHalf, HotaruBufRead, HotaruBufWrite, HotaruIOError, HotaruRead,
    HotaruWrite, MaybeSend, MaybeSendBoxFuture,
};
pub use self::primitive::{Accepter, Connector};
pub use self::runtime::{Inbound, Outbound};
pub use self::stream::{ConnMeta, ConnStream};
pub use self::transport_spec::TransportSpec;
