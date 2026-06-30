pub mod connection;
pub mod error;
pub mod io;
pub mod primitive;
pub mod runtime;
pub mod stream;
pub mod tcp;
pub mod test;
pub mod transport_spec;

pub use self::error::Result;
pub use self::io::buf_reader::HotaruBufReader;
pub use self::io::buf_writer::HotaruBufWriter;
pub use self::io::{
    BufferedReadHalf, BufferedWriteHalf, HotaruBufRead, HotaruBufWrite, HotaruIOError, HotaruRead,
    HotaruWrite, IoCompat,
};
#[cfg(feature = "io_tokio")]
pub use self::io::TokioIo;
#[cfg(feature = "io_embedded")]
pub use self::io::EmbeddedIo;
#[cfg(feature = "io_futures")]
pub use self::io::FuturesIo;
pub use self::primitive::{Accepter, Connector};
pub use self::runtime::{Inbound, Outbound};
pub use self::stream::{ConnMeta, ConnStream};
pub use self::tcp::{
    TcpAccepter, TcpConnector, TcpConnectorAddr, TcpInbound, TcpMeta, TcpOutbound, TcpTransport,
};
pub use self::transport_spec::TransportSpec;
