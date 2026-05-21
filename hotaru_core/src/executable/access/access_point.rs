use std::sync::Arc;

use crate::{
    connection::TransportSpec,
    protocol::RequestContext,
    url::{PathPattern, UrlNode, UrlRegistration},
};

/// One named access point.
///
/// Stores the parsed path the binding was registered at, plus the
/// [`UrlRegistration`] result that points to the live node. Use
/// [`AccessPoint::resolve`] to get the current `UrlNode<C, TS>` — that
/// method transparently follows the root-endpoint slot's indirection for
/// `Root` variants, so root rebinds are picked up automatically.
pub struct AccessPoint<C: RequestContext, TS: TransportSpec> {
    pub path: Vec<PathPattern>,
    pub target: UrlRegistration<C, TS>,
}

impl<C: RequestContext, TS: TransportSpec> AccessPoint<C, TS> {
    /// Resolve to the current `UrlNode`.
    ///
    /// - **`Node` variant** — returns the stored `Arc` directly. The
    ///   `AccessPointTable::refresh_path` mechanism keeps this `Arc` in
    ///   sync with rebinds on the same path.
    /// - **`Root` variant** — reads the live endpoint slot inside the
    ///   `RootNode` via its `PRwLock`. Self-refreshing: a root-endpoint
    ///   rebind doesn't need any explicit notification — the next
    ///   `resolve()` call returns the new endpoint. Returns `None` only if
    ///   the root endpoint slot is empty (registration was never completed).
    pub fn resolve(&self) -> Option<Arc<UrlNode<C, TS>>> {
        match &self.target {
            UrlRegistration::Root(root) => root.endpoint(),
            UrlRegistration::Node(node) => Some(node.clone()),
        }
    }
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
