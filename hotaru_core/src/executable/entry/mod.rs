use std::{
    any::Any,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
    time::Duration,
};

use akari::extensions::{Locals, Params};
use tokio::io::BufReader;

use crate::{
    app::common::RuntimeConfig,
    connection::{ConnStream, TransportSpec},
    protocol::Protocol,
    executable::middleware::AsyncMiddlewareChain,
    url::UrlRoot,
};
use crate::protocol::{Channel, ProtocolFlow};

pub mod builder;

pub use builder::ProtocolEntryBuilder;

/// Neutral protocol-entry boundary shared by server and client execution.
///
/// This trait lives in `executable` rather than under `server` or `client`
/// because the stored entry data is the same on both sides:
/// protocol instance, root URL tree, and protocol-scoped middleware defaults.
/// The role-specific difference is expressed by method families like `serve`
/// and `request`, not by duplicating the entry type itself.
pub trait ProtocolEntryTrait<TS: TransportSpec>: Send + Sync {
    /// Test if this protocol can handle the connection.
    fn test(&self, buf: &[u8]) -> bool;

    /// Handle the connection.
    fn serve(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Handle an upgrade from another protocol.
    fn serve_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        params: RwLock<Params>,
        locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn request(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn request_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        params: RwLock<Params>,
        locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Returns the protocol's default connection-timeout policy.
    ///
    /// Used to resolve [`TimeoutSetting::Inherit`] at connection time.
    fn default_connection_timeout(&self) -> Option<Duration>;

    /// Allows downcasting.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Concrete handler for a specific protocol.
pub struct ProtocolEntry<P, TS>
where
    P: Protocol<Wire = TS::Wire, TS = TS> + Clone,
    TS: TransportSpec,
{
    pub protocol: P,
    pub root_handler: Arc<UrlRoot<P::Context, TS>>,
    pub middlewares: AsyncMiddlewareChain<P::Context>,
}

impl<P, TS> ProtocolEntry<P, TS>
where
    P: Protocol<Wire = TS::Wire, TS = TS> + Clone,
    TS: TransportSpec,
{
    pub fn new(
        protocol: P,
        root_handler: Arc<UrlRoot<P::Context, TS>>,
        middlewares: AsyncMiddlewareChain<P::Context>,
    ) -> Self {
        Self {
            protocol,
            root_handler,
            middlewares,
        }
    }
}

impl<P, TS> ProtocolEntryTrait<TS> for ProtocolEntry<P, TS>
where
    P: Protocol<Wire = TS::Wire, TS = TS> + Clone + 'static,
    TS: TransportSpec,
{
    fn test(&self, buf: &[u8]) -> bool {
        P::detect(buf)
    }

    fn default_connection_timeout(&self) -> Option<Duration> {
        self.protocol.default_connection_timeout()
    }

    fn serve(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let protocol = self.protocol.clone();
        let root = self.root_handler.clone();

        Box::pin(async move {
            let channel = protocol.open_channel(reader, writer, meta);
            while channel.is_open() {
                match P::handle(&channel, runtime.clone(), root.clone()).await {
                    Ok(ProtocolFlow::Continue) => continue,
                    Ok(ProtocolFlow::Close) => {
                        channel.close();
                    }
                    Err(_e) => {
                        channel.close();
                        return;
                    }
                }
            }
        })
    }

    fn serve_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        _params: RwLock<Params>,
        _locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.serve(runtime, reader, writer, meta)
    }

    fn request(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let protocol = self.protocol.clone();
        let root = self.root_handler.clone();

        Box::pin(async move {
            let channel = protocol.open_channel(reader, writer, meta);
            while channel.is_open() {
                match P::handle(&channel, runtime.clone(), root.clone()).await {
                    Ok(ProtocolFlow::Continue) => continue,
                    Ok(ProtocolFlow::Close) => {
                        channel.close();
                    }
                    Err(_e) => {
                        channel.close();
                        return;
                    }
                }
            }
        })
    }

    fn request_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        _params: RwLock<Params>,
        _locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.request(runtime, reader, writer, meta)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
