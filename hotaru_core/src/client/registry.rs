use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use std::sync::OnceLock;

use crate::app::middleware::{AsyncFinalHandler, AsyncMiddleware};
use crate::connection::RequestContext;
use crate::extensions::Params;
use crate::url::PathPattern;

pub type OutpointHandler<C> = Arc<dyn AsyncFinalHandler<C>>;

pub struct OutpointEntry<C: RequestContext> {
    pub patterns: Vec<PathPattern>,
    pub names: Vec<Option<String>>,
    pub handler: OutpointHandler<C>,
    pub middlewares: Vec<Arc<dyn AsyncMiddleware<C>>>,
    pub config: Params,
}

pub struct ClientRegistry {
    entries: RwLock<HashMap<(String, String), Box<dyn Any + Send + Sync>>>,
}

impl ClientRegistry {
    pub fn global() -> &'static ClientRegistry {
        static REGISTRY: OnceLock<ClientRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| ClientRegistry {
            entries: RwLock::new(HashMap::new()),
        })
    }

    pub fn register<C: RequestContext + 'static>(
        &self,
        client_name: impl Into<String>,
        outpoint_name: impl Into<String>,
        entry: OutpointEntry<C>,
    ) {
        let key = (client_name.into(), outpoint_name.into());
        let mut guard = self.entries.write().expect("ClientRegistry lock poisoned");
        guard.insert(key, Box::new(Arc::new(entry)));
    }

    pub fn get<C: RequestContext + 'static>(
        &self,
        client_name: &str,
        outpoint_name: &str,
    ) -> Option<Arc<OutpointEntry<C>>> {
        let key = (client_name.to_string(), outpoint_name.to_string());
        let guard = self.entries.read().expect("ClientRegistry lock poisoned");
        guard
            .get(&key)
            .and_then(|entry| entry.downcast_ref::<Arc<OutpointEntry<C>>>().cloned())
    }
}
