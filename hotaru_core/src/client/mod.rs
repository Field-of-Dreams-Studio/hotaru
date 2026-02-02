use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::app::middleware::{AsyncFinalHandler, AsyncMiddleware};
use crate::connection::{Protocol, RequestContext};
use crate::extensions::{Locals, Params};
use crate::url::{parser::parse, PathPattern};

pub mod registry;
pub mod connection_target;

pub use registry::{ClientRegistry, OutpointEntry, OutpointHandler};
pub use connection_target::ConnectionTarget;

pub struct Client {
    pub name: String,
    pub base_url: Option<String>,
    pub config: Params,
    pub statics: Locals,
    middlewares: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

pub struct ClientBuilder {
    name: Option<String>,
    base_url: Option<String>,
    config: Params,
    statics: Locals,
    middlewares: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            base_url: None,
            config: Params::new(),
            statics: Locals::new(),
            middlewares: HashMap::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn config(mut self, config: Params) -> Self {
        self.config = config;
        self
    }

    pub fn set_config<V: Send + Sync + 'static>(mut self, value: V) -> Self {
        self.config.set(value);
        self
    }

    pub fn statics(mut self, statics: Locals) -> Self {
        self.statics = statics;
        self
    }

    pub fn set_statics<K: Into<String>, V: Send + Sync + 'static>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.statics.set(key, value);
        self
    }

    pub fn middleware<P: Protocol + 'static>(
        mut self,
        mw: Arc<dyn AsyncMiddleware<P::Context>>,
    ) -> Self {
        let entry = self
            .middlewares
            .entry(TypeId::of::<P>())
            .or_insert_with(|| Box::new(Vec::<Arc<dyn AsyncMiddleware<P::Context>>>::new()));
        if let Some(vec) = entry.downcast_mut::<Vec<Arc<dyn AsyncMiddleware<P::Context>>>>() {
            vec.push(mw);
        }
        self
    }

    pub fn middlewares<P: Protocol + 'static>(
        mut self,
        mws: Vec<Arc<dyn AsyncMiddleware<P::Context>>>,
    ) -> Self {
        for mw in mws {
            self = self.middleware::<P>(mw);
        }
        self
    }

    pub fn build(self) -> Arc<Client> {
        Arc::new(Client {
            name: self.name.unwrap_or_else(|| "client".to_string()),
            base_url: self.base_url,
            config: self.config,
            statics: self.statics,
            middlewares: self.middlewares,
        })
    }
}

pub type SClient = Lazy<Arc<Client>>;

pub struct OutpointRegistration<C: RequestContext> {
    client_name: String,
    patterns: Vec<PathPattern>,
    names: Vec<Option<String>>,
    handler: Option<OutpointHandler<C>>,
    middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>,
    config: Params,
}

impl<C: RequestContext> OutpointRegistration<C> {
    fn new(
        client_name: String,
        patterns: Vec<PathPattern>,
        names: Vec<Option<String>>,
    ) -> Self {
        Self {
            client_name,
            patterns,
            names,
            handler: None,
            middlewares: Vec::new(),
            config: Params::new(),
        }
    }

    pub fn set_method(&mut self, handler: Arc<dyn AsyncFinalHandler<C>>) {
        self.handler = Some(handler);
    }

    pub fn set_middlewares(&mut self, middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>) {
        self.middlewares = middlewares;
    }

    pub fn set_params<V: Send + Sync + 'static>(&mut self, value: V) {
        self.config.set(value);
    }

    pub fn register(self, outpoint_name: impl Into<String>) {
        let handler = self.handler.expect("Outpoint handler not set");
        ClientRegistry::global().register(
            self.client_name,
            outpoint_name,
            OutpointEntry {
                patterns: self.patterns,
                names: self.names,
                handler,
                middlewares: self.middlewares,
                config: self.config,
            },
        );
    }
}

impl Client {
    pub fn new() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn config(self: &Arc<Self>) -> &Params {
        &self.config
    }

    pub fn statics(self: &Arc<Self>) -> &Locals {
        &self.statics
    }

    pub fn get_config<T: Clone + Send + Sync + 'static>(self: &Arc<Self>) -> Option<T> {
        self.config.get::<T>().cloned()
    }

    pub fn get_static<T: Clone + Send + Sync + 'static>(self: &Arc<Self>, key: &str) -> Option<T> {
        self.statics.get::<T>(key).cloned()
    }

    pub fn get_config_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
    ) -> T {
        self.config.get::<T>().cloned().unwrap_or_default()
    }

    pub fn get_static_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
        key: &str,
    ) -> T {
        self.statics.get::<T>(key).cloned().unwrap_or_default()
    }

    pub fn get_client_middlewares<P: Protocol + 'static>(
        &self,
    ) -> Vec<Arc<dyn AsyncMiddleware<P::Context>>> {
        self.middlewares
            .get(&TypeId::of::<P>())
            .and_then(|mws| mws.downcast_ref::<Vec<Arc<dyn AsyncMiddleware<P::Context>>>>())
            .cloned()
            .unwrap_or_default()
    }

    pub fn url<P: Protocol, A: AsRef<str>>(
        self: &Arc<Self>,
        path: A,
    ) -> OutpointRegistration<P::Context> {
        match parse(path.as_ref()) {
            Ok((patterns, names)) => {
                OutpointRegistration::new(self.name.clone(), patterns, names)
            }
            Err(e) => {
                crate::debug_error!("Error parsing outpoint URL: {}", e);
                OutpointRegistration::new(self.name.clone(), Vec::new(), Vec::new())
            }
        }
    }
}
