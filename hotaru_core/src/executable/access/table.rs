#[cfg(not(feature = "std"))]
use crate::prelude::*;
use alloc::sync::Arc;
use akari::hash::HashMap;

use crate::{
    alias::PRwLock,
    connection::TransportSpec,
    protocol::RequestContext,
    url::{PathPattern, UrlNode, UrlRegistration},
};

use super::access_point::AccessPoint;

/// Named lookup table of [`AccessPoint`]s, scoped to one `ProtocolEntry`.
///
/// The table maps a user-chosen string name to the access point that holds
/// the path it was registered at plus a `UrlRegistration` pointing at the
/// current node. See [`AccessPoint::resolve`] for how `Root` vs `Node`
/// variants are followed.
///
/// Lookup is `O(1)` average via the inner `HashMap`. `refresh_path` is a
/// linear scan because it must find every entry whose stored path equals
/// the rebound path — that scan is only triggered at registration time, so
/// the cost is paid once per registration, not per request.
///
/// # TODO(perf): single coarse lock
///
/// All operations on this table currently serialize on **one** `RwLock`
/// guarding the whole `HashMap`. This is fine while:
///
/// - Registrations happen at startup (no write contention at steady state).
/// - Lookup QPS is moderate (multiple readers can hold the `RwLock`
///   concurrently, but write requests still block all readers).
///
/// On a per-request hot path (`Client::request_fn(name, ...)` resolves a
/// name on every outbound call), the read lock acquisition becomes a
/// measurable cost as concurrency scales. When that bites, consider:
///
/// - **Sharded locking** (`dashmap` or hand-rolled) — partitions the
///   keyspace so different names don't contend.
///   Refrain from using DashMap directly. 
/// - **Copy-on-write `Arc<HashMap>` + atomic swap on write** — best fit
///   here since registrations are rare and reads dominate. Lookups become
///   lock-free (just an Arc load).
/// - **Structural concurrent map** (e.g. `flurry`) — lock-free reads and
///   writes; bigger dependency.
///
/// Any replacement should preserve the `refresh_path` semantics (scan all
/// entries with matching path, refresh `Node` variants only) — a sharded
/// map needs cross-shard iteration support, which `dashmap` provides.
pub struct AccessPointTable<C: RequestContext, TS: TransportSpec> {
    inner: PRwLock<HashMap<String, AccessPoint<C, TS>>>,
}

impl<C: RequestContext, TS: TransportSpec> AccessPointTable<C, TS> {
    pub fn new() -> Self {
        Self { inner: PRwLock::new(HashMap::default()) }
    }

    /// Insert a named access point, returning the previous entry under that
    /// name if any.
    pub fn insert<N: Into<String>>(
        &self,
        name: N,
        ap: AccessPoint<C, TS>,
    ) -> Option<AccessPoint<C, TS>> {
        self.inner.write().insert(name.into(), ap)
    }

    /// Refresh the stored node for every **Node-variant** entry whose path
    /// equals `path`. Root-variant entries are skipped — their `RootNode`
    /// indirection picks up rebinds automatically via `AccessPoint::resolve`.
    /// Returns the number of entries refreshed.
    pub fn refresh_path(
        &self,
        path: &[PathPattern],
        node: &Arc<UrlNode<C, TS>>,
    ) -> usize {
        let mut guard = self.inner.write();
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

    /// Look up a named access point. Returns a cloned snapshot; callers
    /// should call `resolve()` on the result to obtain the current node.
    pub fn get(&self, name: &str) -> Option<AccessPoint<C, TS>> {
        self.inner.read().get(name).cloned()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.inner.read().contains_key(name)
    }

    pub fn remove(&self, name: &str) -> Option<AccessPoint<C, TS>> {
        self.inner.write().remove(name)
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Returns all registered names. Allocated; useful for introspection
    /// and tests, not the hot path.
    pub fn names(&self) -> Vec<String> {
        self.inner.read().keys().cloned().collect()
    }
}

impl<C: RequestContext, TS: TransportSpec> Default for AccessPointTable<C, TS> {
    fn default() -> Self {
        Self::new()
    }
}
