use std::{collections::HashMap, sync::Arc};
use std::sync::RwLock;

use crate::{connection::TransportSpec, protocol::RequestContext, url::{PathPattern, UrlNode, UrlRegistration}};

pub struct AccessPoint<C: RequestContext, TS: TransportSpec> {
    pub path: Vec<PathPattern>,
    pub target: UrlRegistration<C, TS>,
}

impl<C: RequestContext, TS: TransportSpec> Clone for AccessPoint<C, TS> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            target: match &self.target {
                UrlRegistration::Root(r) => UrlRegistration::Root(r.clone()),
                UrlRegistration::Node(n) => UrlRegistration::Node(n.clone()),
            },
        }
    }
}

pub struct AccessPoints<C: RequestContext, TS: TransportSpec> {
    inner: RwLock<HashMap<String, AccessPoint<C, TS>>>,
}

impl<C: RequestContext, TS: TransportSpec> AccessPoints<C, TS> {
    pub fn new() -> Self {
        Self { inner: RwLock::new(HashMap::new()) }
    }

    pub fn insert<N: Into<String>>(
        &self,
        name: N,
        ap: AccessPoint<C, TS>,
    ) -> Option<AccessPoint<C, TS>> {
        self.inner.write().unwrap().insert(name.into(), ap)
    }

    /// Refresh the node for every **Node-variant** entry whose stored path
    /// equals `path`. Root-variant entries are skipped — their PRwLock
    /// indirection makes them self-refreshing. Returns the number of
    /// entries refreshed.
    pub fn refresh_path(
        &self,
        path: &[PathPattern],
        node: &Arc<UrlNode<C, TS>>,
    ) -> usize {
        let mut guard = self.inner.write().unwrap();
        let mut count = 0;
        for ap in guard.values_mut() {
            if ap.path.as_slice() == path {
                if let UrlRegistration::Node(slot) = &mut ap.target {
                    *slot = node.clone();
                    count += 1;
                }
                // Root variants intentionally skipped.
            }
        }
        count
    }

    pub fn get(&self, name: &str) -> Option<AccessPoint<C, TS>> {
        self.inner.read().unwrap().get(name).cloned()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.inner.read().unwrap().contains_key(name)
    }

    pub fn remove(&self, name: &str) -> Option<AccessPoint<C, TS>> {
        self.inner.write().unwrap().remove(name)
    }

    pub fn len(&self) -> usize { self.inner.read().unwrap().len() }
    pub fn is_empty(&self) -> bool { self.inner.read().unwrap().is_empty() }
    pub fn names(&self) -> Vec<String> {
        self.inner.read().unwrap().keys().cloned().collect()
    }
} 
