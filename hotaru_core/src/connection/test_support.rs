//! Test-only dummy transport for protocol/url type tests.

use core::convert::Infallible;
use core::net::SocketAddr;

use crate::connection::{
    ConnMeta, ConnStream, HotaruBufRead, HotaruBufWrite, HotaruRead, HotaruWrite, Inbound,
    Outbound, TransportSpec,
};

pub enum TestWire {}

pub struct TestMeta;

impl ConnMeta for TestMeta {}

impl HotaruRead for TestWire {
    type Error = Infallible;
    type Buffered = Self;

    fn into_buf(self) -> Self::Buffered {
        match self {}
    }

    async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        match *self {}
    }

    async fn read_exact(&mut self, _buf: &mut [u8]) -> Result<(), Self::Error> {
        match *self {}
    }
}

impl HotaruBufRead for TestWire {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        match *self {}
    }

    fn consume(&mut self, _amt: usize) {
        match *self {}
    }
}

impl HotaruWrite for TestWire {
    type Error = Infallible;
    type Buffered = Self;

    fn into_buf_write(self) -> Self::Buffered {
        match self {}
    }

    async fn write(&mut self, _buf: &[u8]) -> Result<usize, Self::Error> {
        match *self {}
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        match *self {}
    }

    async fn write_all(&mut self, _buf: &[u8]) -> Result<(), Self::Error> {
        match *self {}
    }
}

impl HotaruBufWrite for TestWire {}

impl ConnStream for TestWire {
    type ReadHalf = Self;
    type WriteHalf = Self;
    type Meta = TestMeta;

    fn split(self) -> (Self::ReadHalf, Self::WriteHalf, Self::Meta) {
        match self {}
    }

    fn peer_addr(&self) -> Option<SocketAddr> {
        match *self {}
    }

    fn local_addr(&self) -> Option<SocketAddr> {
        match *self {}
    }
}

pub struct TestInbound;

impl Inbound for TestInbound {
    type Wire = TestWire;
    type BindTarget = ();
    type Error = Infallible;

    async fn bind(_target: Self::BindTarget) -> Result<Self, Self::Error> {
        panic!("TestInbound is a type-only test stub")
    }

    async fn accept(&self) -> Result<Self::Wire, Self::Error> {
        panic!("TestInbound is a type-only test stub")
    }
}

pub struct TestOutbound;

impl Outbound for TestOutbound {
    type Wire = TestWire;
    type ConnectTarget = ();
    type Error = Infallible;

    async fn build(_target: Self::ConnectTarget) -> Result<Self, Self::Error> {
        panic!("TestOutbound is a type-only test stub")
    }

    async fn connect(&self) -> Result<Self::Wire, Self::Error> {
        panic!("TestOutbound is a type-only test stub")
    }
}

pub struct TestTransport;

impl TransportSpec for TestTransport {
    type Wire = TestWire;
    type IoError = Infallible;
    type Inbound = TestInbound;
    type Outbound = TestOutbound;
}
