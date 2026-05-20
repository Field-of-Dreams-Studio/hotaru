use std::{any::TypeId, sync::Arc};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{
    app::common::RuntimeConfig,
    connection::{ConnStream, TransportSpec},
    debug_log,
    executable::{
        ExecutableBinding,
        entry::{ProtocolEntry, ProtocolEntryTrait},
        middleware::{AsyncMiddleware, AsyncMiddlewareChain},
    },
    extensions::ParamsClone,
    protocol::Protocol,
    url::{UrlError, UrlRegistration, UrlRoot},
};

pub mod builder;

pub use builder::ProtocolRegistryBuilder;

/// Registry for multiple protocol entries.
pub struct ProtocolEntryRegistry<TS: TransportSpec> {
    pub(crate) handlers: Vec<Arc<dyn ProtocolEntryTrait<TS>>>,
}

impl<TS: TransportSpec> ProtocolEntryRegistry<TS> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register a protocol entry.
    pub fn register<P: Protocol<Wire = TS::Wire, TS = TS> + Clone + 'static>(
        &mut self,
        protocol: P,
        root_handler: Arc<UrlRoot<P::Context, TS>>,
        middleware_chain: AsyncMiddlewareChain<P::Context>,
    ) {
        self.handlers.push(Arc::new(ProtocolEntry::new(
            protocol,
            root_handler,
            middleware_chain,
        )));
    }

    pub async fn serve(&self, runtime: Arc<RuntimeConfig>, conn: TS::Wire) {
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
            handler.serve(runtime, reader, writer, meta).await;
        } else {
            let _ = writer.shutdown().await;
        }
    }

    pub async fn request(&self, runtime: Arc<RuntimeConfig>, conn: TS::Wire) {
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
            handler.request(runtime, reader, writer, meta).await;
        } else {
            let _ = writer.shutdown().await;
        }
    }

    pub fn url<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        &self,
    ) -> Option<Arc<UrlRoot<P::Context, TS>>> {
        for handler in &self.handlers {
            if let Some(ph) = handler.as_any().downcast_ref::<ProtocolEntry<P, TS>>() {
                return Some(ph.root_handler.clone());
            }
        }
        None
    }

    pub fn lit_url<P: Protocol<Wire = TS::Wire, TS = TS> + 'static, T: Into<String>>(
        &self,
        url: T,
        executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<UrlRegistration<P::Context, TS>, UrlError> {
        let url = url.into();
        match self
            .url::<P>()
            .map(|root| root.literal_url(&url, executable, config))
        {
            Some(result) => result,
            None => Err(UrlError::ProtocolNotFound),
        }
    }

    pub fn sub_url<P: Protocol<Wire = TS::Wire, TS = TS> + 'static, T: Into<String>>(
        &self,
        pattern: T,
        executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<UrlRegistration<P::Context, TS>, UrlError> {
        let pattern = pattern.into();
        match self
            .url::<P>()
            .map(|root| root.sub_url(&pattern, executable, config))
        {
            Some(result) => result,
            None => Err(UrlError::ProtocolNotFound),
        }
    }

    pub fn first_protocol_type_id(&self) -> Option<TypeId> {
        self.handlers.first().map(|h| h.as_any().type_id())
    }

    pub fn get_protocol_middlewares<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        &self,
    ) -> Vec<Arc<dyn AsyncMiddleware<P::Context>>> {
        for handler in &self.handlers {
            if let Some(protocol_entry) = handler.as_any().downcast_ref::<ProtocolEntry<P, TS>>() {
                return protocol_entry.middlewares.clone();
            }
        }
        vec![]
    }
}
