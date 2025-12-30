use super::application::App;
use crate::{
    app::middleware::{AsyncMiddleware, AsyncMiddlewareChain},
    connection::{
        Protocol, ProtocolRole, TcpConnectionStream, TcpReader, TcpWriter, split_connection,
    },
    debug_log, debug_error,
    extensions::ParamsClone,
    url::{PathPattern, Url},
};
use akari::extensions::{Locals, Params};
use std::{
    any::{Any, TypeId},
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
};
use tokio::io::AsyncBufReadExt;

/// Concrete handler for a specific protocol
struct ProtocolHandler<P: Protocol + Clone> {
    protocol: P,
    root_handler: Arc<Url<P::Context>>,
    middlewares: AsyncMiddlewareChain<P::Context>,
}

impl<P: Protocol + Clone> ProtocolHandler<P> {
    pub fn new(protocol: P, root_handler: Arc<Url<P::Context>>, middlewares: AsyncMiddlewareChain<P::Context>) -> Self {
        Self {
            protocol,
            root_handler,
            middlewares,
        }
    }
}

pub trait ProtocolHandlerTrait: Send + Sync {
    /// Test if this protocol can handle the connection
    fn test(&self, buf: &[u8]) -> bool;

    /// Handle the connection
    fn handle(
        &self,
        app: Arc<App>,
        reader: TcpReader,
        writer: TcpWriter,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Handle an upgrade from another protocol
    fn handle_upgrade(
        &self,
        app: Arc<App>,
        reader: TcpReader,
        writer: TcpWriter,
        params: RwLock<Params>,
        locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Allows downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Attach the application reference
    fn attach_app(&self, app: Arc<App>);
}

impl<P: Protocol + Clone + 'static> ProtocolHandlerTrait for ProtocolHandler<P> {
    fn test(&self, buf: &[u8]) -> bool {
        P::detect(buf)
    }

    fn handle(
        &self,
        app: Arc<App>,
        reader: TcpReader,
        writer: TcpWriter,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let mut protocol = self.protocol.clone();

        Box::pin(async move {
            // Pass buffered readers directly to preserve peeked data
            if let Err(e) = protocol.handle(reader, writer, app).await {
                debug_error!("Protocol error: {}", e);
            }
        })
    }

    fn handle_upgrade(
        &self,
        app: Arc<App>,
        reader: TcpReader,
        writer: TcpWriter,
        _params: RwLock<Params>,
        _locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.handle(app, reader, writer)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn attach_app(&self, app: Arc<App>) {
        self.root_handler.set_app(app);
    }
}

/// Registry for multiple protocol handlers
pub struct ProtocolRegistry {
    handlers: Vec<Arc<dyn ProtocolHandlerTrait>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register a protocol
    pub fn register<P: Protocol + Clone + 'static>(
        &mut self,
        protocol: P,
        root_handler: Arc<Url<P::Context>>,
        middleware_chain: AsyncMiddlewareChain<P::Context>,
    ) {
        self.handlers.push(Arc::new(ProtocolHandler::new(
            protocol,
            root_handler,
            middleware_chain,
        )));
    }

    /// Run protocol detection and handling
    pub async fn run_multi(&self, app: Arc<App>, conn: TcpConnectionStream) {
        let (mut reader, mut writer) = split_connection(conn);

        // Peek at initial bytes
        let buf = reader.fill_buf().await.unwrap_or(&[]);
        let n = buf.len();

        debug_log!("Protocol detection: {} bytes: {:?}", n, String::from_utf8_lossy(&buf[..n.min(50)]));

        // Find matching protocol
        for (i, handler) in self.handlers.iter().enumerate() {
            debug_log!("Testing protocol handler #{}", i);
            if handler.test(&buf[..n]) {
                debug_log!("Protocol #{} matched!", i);
                handler.handle(app.clone(), reader, writer).await;
                return;
            }
        }

        debug_log!("No protocol matched!");

        // No protocol matched
        let _ = writer.shutdown().await;
    }
}

/// Protocol registry modes
pub enum ProtocolRegistryKind {
    Single(Arc<dyn ProtocolHandlerTrait>),
    Multi(ProtocolRegistry),
}

/// Builder for protocol handlers
pub struct ProtocolHandlerBuilder<P: Protocol + Clone + 'static> {
    protocol: Option<P>,
    url: Arc<Url<P::Context>>,
    middlewares: Vec<Arc<dyn AsyncMiddleware<P::Context>>>,
}

impl<P: Protocol + Clone> ProtocolHandlerBuilder<P> {
    pub fn new(protocol: P) -> Self {
        Self {
            protocol: Some(protocol),
            url: Arc::new(Url::default()),
            middlewares: Vec::new(),
        }
    }

    pub fn set_url(mut self, url: Arc<Url<P::Context>>) -> Self {
        self.url = url;
        self
    }

    pub fn append_middleware<M>(mut self) -> Self
    where
        M: AsyncMiddleware<P::Context> + 'static,
    {
        self.middlewares.push(Arc::new(M::return_self()));
        self
    }

    pub fn build(self) -> Arc<dyn ProtocolHandlerTrait> {
        Arc::new(ProtocolHandler::new(
            self.protocol.expect("Protocol must be set"),
            self.url,
            self.middlewares,
        ))
    }
}

pub struct ProtocolRegistryBuilder {
    handlers: Vec<Arc<dyn ProtocolHandlerTrait>>,
}

impl ProtocolRegistryBuilder {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn protocol<P: Protocol + Clone>(mut self, builder: ProtocolHandlerBuilder<P>) -> Self {
        self.handlers.push(builder.build());
        self
    }

    pub fn build(self) -> ProtocolRegistryKind {
        match self.handlers.len() {
            1 => ProtocolRegistryKind::Single(self.handlers.into_iter().next().unwrap()),
            _ => ProtocolRegistryKind::Multi(ProtocolRegistry {
                handlers: self.handlers,
            }),
        }
    }
}

impl ProtocolRegistryKind {
    pub fn single<P: Protocol + Clone + 'static>(
        protocol: P,
        root_handler: Arc<Url<P::Context>>,
        middlewares: AsyncMiddlewareChain<P::Context>,
    ) -> Self {
        ProtocolRegistryKind::Single(Arc::new(ProtocolHandler::new(protocol, root_handler, middlewares)))
    }

    pub fn multi(registry: ProtocolRegistry) -> Self {
        ProtocolRegistryKind::Multi(registry)
    }

    pub fn attach_app(&self, app: Arc<App>) {
        match self {
            ProtocolRegistryKind::Single(handler) => handler.attach_app(app),
            ProtocolRegistryKind::Multi(registry) => {
                for handler in &registry.handlers {
                    handler.attach_app(app.clone());
                }
            }
        }
    }

    pub async fn run(&self, app: Arc<App>, conn: TcpConnectionStream) {
        match self {
            ProtocolRegistryKind::Single(handler) => {
                let (reader, writer) = split_connection(conn);
                handler.handle(app, reader, writer).await;
            }
            ProtocolRegistryKind::Multi(registry) => {
                registry.run_multi(app, conn).await;
            }
        }
    }

    pub fn url<P: Protocol + 'static>(&self) -> Option<Arc<Url<P::Context>>> {
        match self {
            ProtocolRegistryKind::Single(handler) => handler
                .as_any()
                .downcast_ref::<ProtocolHandler<P>>()
                .map(|ph| ph.root_handler.clone()),
            ProtocolRegistryKind::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(ph) = handler.as_any().downcast_ref::<ProtocolHandler<P>>() {
                        return Some(ph.root_handler.clone());
                    }
                }
                None
            }
        }
    }

    pub fn lit_url<P: Protocol + 'static, T: Into<String>>(&self, url: T) -> Result<Arc<Url<P::Context>>, String> {
        let url = url.into();
        match self.url::<P>().map(|root| {
            root.clone().literal_url(
                &url,
                None,
                None,
                ParamsClone::default(),
            )
        }) {
            Some(Ok(url)) => Ok(url),
            Some(Err(e)) => Err(e),
            None => Err("Protocol Not Found".to_string()),
        }
    }

    pub fn sub_url<P: Protocol + 'static, A: AsRef<str>>(
        &self,
        pattern: A,
    ) -> Result<Arc<Url<P::Context>>, String> {
        match self.url::<P>().map(|root| {
            root.clone().sub_url(
                pattern,
                None,
                None,
                ParamsClone::default(),
            )
        }) {
            Some(Ok(url)) => Ok(url),
            Some(Err(e)) => Err(e),
            None => Err("Protocol Not Found".to_string()),
        }
    }

    // TODO: Implement register_from on Url or remove this method
    // pub fn reg_from<P: Protocol + 'static>(&self, segments: &[PathPattern]) -> Result<Arc<Url<P::Context>>, String> {
    //     match self.url::<P>().map(|root| {
    //         root.clone().register_from(segments.to_vec())
    //     }) {
    //         Some(url) => Ok(url),
    //         None => Err("Protocol Not Found".to_string()),
    //     }
    // }

    pub fn first_protocol_type_id(&self) -> Option<TypeId> {
        match self {
            ProtocolRegistryKind::Single(handler) => Some(handler.as_any().type_id()),
            ProtocolRegistryKind::Multi(registry) => {
                registry.handlers.first().map(|h| h.as_any().type_id())
            }
        }
    }

    /// Get protocol-level middleware for inheritance
    /// Returns middleware from the specified protocol type for use in endpoint inheritance
    pub fn get_protocol_middlewares<P: Protocol + 'static>(&self) -> Vec<Arc<dyn AsyncMiddleware<P::Context>>> {
        match self {
            ProtocolRegistryKind::Single(handler) => {
                // Try to downcast to the specific protocol handler
                if let Some(protocol_handler) = handler.as_any().downcast_ref::<ProtocolHandler<P>>() {
                    protocol_handler.middlewares.clone()
                } else {
                    vec![]
                }
            }
            ProtocolRegistryKind::Multi(registry) => {
                // Find the matching protocol handler
                for handler in &registry.handlers {
                    if let Some(protocol_handler) = handler.as_any().downcast_ref::<ProtocolHandler<P>>() {
                        return protocol_handler.middlewares.clone();
                    }
                }
                vec![]
            }
        }
    }
}