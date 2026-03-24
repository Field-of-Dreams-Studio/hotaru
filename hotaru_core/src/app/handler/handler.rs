use std::{
    any::Any,
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};

use akari::extensions::{Locals, Params};
use tokio::io::BufReader;

use crate::{
    executable::middleware::AsyncMiddlewareChain,
    app::application::App,
    connection::{ConnStream, Protocol, TransportSpec},
    debug_error,
    url::Url,
};

/// Trait-object boundary for protocol handlers used by the app registry.
pub trait ProtocolHandlerTrait<TS: TransportSpec>: Send + Sync {
    /// Test if this protocol can handle the connection.
    fn test(&self, buf: &[u8]) -> bool;

    /// Handle the connection.
    fn handle(
        &self,
        app: Arc<App<TS>>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Handle an upgrade from another protocol.
    fn handle_upgrade(
        &self,
        app: Arc<App<TS>>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        params: RwLock<Params>,
        locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Allows downcasting.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Attach the application reference.
    fn attach_app(&self, app: Arc<App<TS>>);
}

/// Concrete handler for a specific protocol.
pub struct ProtocolHandler<P, TS>
where
    P: Protocol + Clone,
    TS: TransportSpec<Wire = P::Wire>,
{
    pub protocol: P,
    pub root_handler: Arc<Url<P::Context, TS>>,
    pub middlewares: AsyncMiddlewareChain<P::Context>,
}

impl<P, TS> ProtocolHandler<P, TS>
where
    P: Protocol + Clone,
    TS: TransportSpec<Wire = P::Wire>,
{
    pub fn new(
        protocol: P,
        root_handler: Arc<Url<P::Context, TS>>,
        middlewares: AsyncMiddlewareChain<P::Context>,
    ) -> Self {
        Self {
            protocol,
            root_handler,
            middlewares,
        }
    }
}

impl<P, TS> ProtocolHandlerTrait<TS> for ProtocolHandler<P, TS>
where
    P: Protocol + Clone + 'static,
    TS: TransportSpec<Wire = P::Wire>,
{
    fn test(&self, buf: &[u8]) -> bool {
        P::detect(buf)
    }

    fn handle(
        &self,
        app: Arc<App<TS>>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let mut protocol = self.protocol.clone();

        Box::pin(async move {
            if let Err(_e) = protocol.handle(reader, writer, meta, app).await {
                debug_error!("Protocol error: {}", _e);
            }
        })
    }

    fn handle_upgrade(
        &self,
        app: Arc<App<TS>>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        _params: RwLock<Params>,
        _locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.handle(app, reader, writer, meta)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn attach_app(&self, _app: Arc<App<TS>>) {
        // Runtime ownership is being removed from Url temporarily.
        // Route trees remain attached structurally, but no longer cache App here.
        // self.root_handler.set_app(app);
    }
}
