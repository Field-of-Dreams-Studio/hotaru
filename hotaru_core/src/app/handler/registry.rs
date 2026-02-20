use super::{ProtocolHandler, ProtocolHandlerTrait};
use crate::{
    app::{application::App, middleware::{AsyncMiddleware, AsyncMiddlewareChain}},
    connection::{ConnStream, Protocol, TransportSpec},
    debug_log,
    extensions::ParamsClone,
    url::Url,
};
use std::{any::TypeId, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Registry for multiple protocol handlers.
pub struct ProtocolRegistry<TS: TransportSpec> {
    pub(crate) handlers: Vec<Arc<dyn ProtocolHandlerTrait<TS>>>,
}

impl<TS: TransportSpec> ProtocolRegistry<TS> {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    /// Register a protocol.
    pub fn register<P: Protocol<Wire = TS::Wire> + Clone + 'static>(
        &mut self,
        protocol: P,
        root_handler: Arc<Url<P::Context, TS>>,
        middleware_chain: AsyncMiddlewareChain<P::Context>,
    ) {
        self.handlers.push(Arc::new(ProtocolHandler::new(
            protocol,
            root_handler,
            middleware_chain,
        )));
    }

    pub async fn run_multi(
        &self,
        app: Arc<App<TS>>,
        conn: TS::Wire,
    ) {
        let (read_half, mut writer, meta) = conn.split();
        let mut reader = BufReader::new(read_half);
        let selected = {
            let buf = reader.fill_buf().await.unwrap_or(&[]);
            let _n = buf.len();
            debug_log!(
                "Protocol detection: {} bytes: {:?}",
                _n,
                String::from_utf8_lossy(&buf[.._n.min(50)])
            );
            self.handlers.iter().find(|h| h.test(buf)).cloned()
        };

        if let Some(handler) = selected {
            handler.handle(app, reader, writer, meta).await;
        } else {
            let _ = writer.shutdown().await;
        }
    }
}

/// Protocol registry modes.
pub enum ProtocolRegistryKind<TS: TransportSpec> {
    Single(Arc<dyn ProtocolHandlerTrait<TS>>),
    Multi(ProtocolRegistry<TS>),
}

impl<TS: TransportSpec> ProtocolRegistryKind<TS> {
    pub fn single<P: Protocol<Wire = TS::Wire> + Clone + 'static>(
        protocol: P,
        root_handler: Arc<Url<P::Context, TS>>,
        middlewares: AsyncMiddlewareChain<P::Context>,
    ) -> Self {
        Self::Single(Arc::new(ProtocolHandler::new(
            protocol,
            root_handler,
            middlewares,
        )))
    }

    pub fn multi(registry: ProtocolRegistry<TS>) -> Self {
        Self::Multi(registry)
    }

    pub fn attach_app(&self, app: Arc<App<TS>>) {
        match self {
            Self::Single(handler) => handler.attach_app(app),
            Self::Multi(registry) => {
                for handler in &registry.handlers {
                    handler.attach_app(app.clone());
                }
            }
        }
    }

    pub async fn run(
        &self,
        app: Arc<App<TS>>,
        conn: TS::Wire,
    ) {
        match self {
            Self::Single(handler) => {
                let (reader, writer, meta) = conn.split();
                let reader = BufReader::new(reader);
                handler.handle(app, reader, writer, meta).await;
            }
            Self::Multi(registry) => {
                registry.run_multi(app, conn).await;
            }
        }
    }

    pub fn url<P: Protocol<Wire = TS::Wire> + 'static>(
        &self,
    ) -> Option<Arc<Url<P::Context, TS>>> {
        match self {
            Self::Single(handler) => handler
                .as_any()
                .downcast_ref::<ProtocolHandler<P, TS>>()
                .map(|ph| ph.root_handler.clone()),
            Self::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(ph) = handler.as_any().downcast_ref::<ProtocolHandler<P, TS>>() {
                        return Some(ph.root_handler.clone());
                    }
                }
                None
            }
        }
    }

    pub fn lit_url<P: Protocol<Wire = TS::Wire> + 'static, T: Into<String>>(
        &self,
        url: T,
    ) -> Result<Arc<Url<P::Context, TS>>, String> {
        let url = url.into();
        match self
            .url::<P>()
            .map(|root| root.clone().literal_url(&url, None, None, ParamsClone::default()))
        {
            Some(Ok(url)) => Ok(url),
            Some(Err(e)) => Err(e),
            None => Err("Protocol Not Found".to_string()),
        }
    }

    pub fn sub_url<P: Protocol<Wire = TS::Wire> + 'static, A: AsRef<str>>(
        &self,
        pattern: A,
    ) -> Result<Arc<Url<P::Context, TS>>, String> {
        match self.url::<P>().map(|root| {
            root.clone()
                .sub_url(pattern, None, None, ParamsClone::default())
        }) {
            Some(Ok(url)) => Ok(url),
            Some(Err(e)) => Err(e),
            None => Err("Protocol Not Found".to_string()),
        }
    }

    pub fn first_protocol_type_id(&self) -> Option<TypeId> {
        match self {
            Self::Single(handler) => Some(handler.as_any().type_id()),
            Self::Multi(registry) => registry.handlers.first().map(|h| h.as_any().type_id()),
        }
    }

    pub fn get_protocol_middlewares<P: Protocol<Wire = TS::Wire> + 'static>(
        &self,
    ) -> Vec<Arc<dyn AsyncMiddleware<P::Context>>> {
        match self {
            Self::Single(handler) => {
                if let Some(protocol_handler) =
                    handler.as_any().downcast_ref::<ProtocolHandler<P, TS>>()
                {
                    protocol_handler.middlewares.clone()
                } else {
                    vec![]
                }
            }
            Self::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(protocol_handler) =
                        handler.as_any().downcast_ref::<ProtocolHandler<P, TS>>()
                    {
                        return protocol_handler.middlewares.clone();
                    }
                }
                vec![]
            }
        }
    }
}
