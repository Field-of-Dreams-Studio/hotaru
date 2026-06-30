use core::slice::Iter;
use alloc::sync::Arc;

use akari::extensions::ParamsClone;

use crate::{
    alias::PRwLock,
    connection::TransportSpec,
    debug_log,
    executable::ExecutableBinding,
    marker::MaybeSendBoxFuture,
    protocol::RequestContext,
    url::{Children, PathPattern, UrlError},
};

use super::{
    node::{PartialState, StepName, UrlNode},
    parser::parse,
};

/// Segmentless dispatch table at the root of a URL tree.
///
/// The root does not represent any path segment itself. Its children are the
/// first-level nodes of the tree. A separate `endpoint` slot holds the handler
/// for the literal empty-string path `""` only — protocols that have a
/// meaningful empty topic (e.g. MQTT) store their root handler here.
///
/// HTTP `"/"` is NOT the root endpoint. It is registered as a two-level
/// `[Literal(""), Literal("")]` tree node so that `"/"` and `""` remain
/// distinct routes.
pub struct RootNode<C: RequestContext, TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    children: Children<C, TS>,
    endpoint: PRwLock<Option<Arc<UrlNode<C, TS>>>>,
}

impl<C: RequestContext + Send + 'static, TS: TransportSpec> RootNode<C, TS> {
    fn new() -> Self {
        Self {
            children: Children::new(),
            endpoint: PRwLock::new(None),
        }
    }

    /// Returns `true` if the root endpoint slot has a compiled handler.
    pub(crate) fn has_handler(&self) -> bool {
        self.endpoint
            .read()
            .as_ref()
            .map_or(false, |n| n.has_handler())
    }

    /// Crate-visible borrow of the root's child table (used by `WalkCursor`).
    pub(crate) fn children_ref(&self) -> &Children<C, TS> {
        &self.children
    }

    /// Returns a cloned `Arc` of the current root endpoint node, if the
    /// slot is populated. Used by [`AccessPoint::resolve`] to follow the
    /// `PRwLock` indirection for `UrlRegistration::Root` entries — every
    /// call reads the current slot, so root-endpoint rebinds are picked
    /// up automatically without explicit `refresh_path` notifications.
    pub fn endpoint(&self) -> Option<Arc<UrlNode<C, TS>>> {
        self.endpoint.read().clone()
    }

    /// Walks from the root's children using a segment iterator.
    ///
    /// If the iterator is exhausted on entry the root endpoint is returned.
    fn walk<'a>(
        self: Arc<Self>,
        mut path: Iter<'a, &str>,
    ) -> MaybeSendBoxFuture<'a, Option<Arc<UrlNode<C, TS>>>> {
        let this_segment = match path.next() {
            Some(s) => *s,
            None => {
                let endpoint = self.endpoint.read().clone();
                return Box::pin(async move { endpoint });
            }
        };

        Box::pin(async move {
            let mut state = PartialState::NotStart;

            while !state.is_end() {
                let (matched_child, next_state) = self.children.match_step(this_segment, state);
                state = next_state;

                let Some(child) = matched_child else {
                    continue;
                };

                if path.len() >= 1 && !child.path().is_any_path() {
                    if let Some(result) = child.clone().walk(path.clone(), PartialState::NotStart).await {
                        return Some(result);
                    }
                } else {
                    return Some(child);
                }
            }

            None
        })
    }
}

/// Outcome of a route registration on a [`UrlRoot`].
///
/// - `Root` — the path was `""` (empty string); the binding was stored in the
///   segmentless root endpoint slot.  Only protocols that have a meaningful
///   empty-path concept (e.g. MQTT) use this variant.
/// - `Node` — the path resolved to a tree node (includes HTTP `"/"`).
pub enum UrlRegistration<
    C: RequestContext,
    TS: TransportSpec = crate::connection::tcp::TcpTransport,
> {
    Root(Arc<RootNode<C, TS>>),
    Node(Arc<UrlNode<C, TS>>),
} 

impl<C: RequestContext + Send + 'static, TS: TransportSpec> Clone for UrlRegistration<C, TS> { 
    fn clone(&self) -> Self {
        match self {
            UrlRegistration::Root(root) => UrlRegistration::Root(root.clone()),
            UrlRegistration::Node(node) => UrlRegistration::Node(node.clone()),
        }
    }
} 

/// Root wrapper for a URL tree.
///
/// The root itself is segmentless — it is a pure dispatch table. Its children
/// are the first-level URL nodes. Protocol handlers are responsible for any
/// protocol-specific normalisation before calling `walk_str` or `walk_str_with_limit`.
///
/// # Root endpoint vs. tree nodes
///
/// `walk_str("")` returns the root endpoint (registered via `literal_url("")`).
/// `walk_str("/")` walks the tree as `["", ""]` — it is a distinct node, never
/// the root endpoint. Only the literal empty-string path maps to the root.
pub struct UrlRoot<C: RequestContext, TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    root: Arc<RootNode<C, TS>>,
}

impl<C: RequestContext + Send + 'static, TS: TransportSpec> UrlRoot<C, TS> {
    /// Creates an empty URL root with no children and no root endpoint.
    pub fn new() -> Self {
        Self {
            root: Arc::new(RootNode::new()),
        }
    }

    /// Walks the URL tree using a segment iterator.
    pub fn walk<'a>(
        &self,
        path: Iter<'a, &str>,
    ) -> MaybeSendBoxFuture<'a, Option<Arc<UrlNode<C, TS>>>> {
        self.root.clone().walk(path)
    }

    /// Walks the URL tree using a segment iterator, rejecting paths deeper than `max_depth`.
    pub fn walk_with_limit<'a>(
        &self,
        path: Iter<'a, &str>,
        max_depth: u32,
    ) -> MaybeSendBoxFuture<'a, Option<Arc<UrlNode<C, TS>>>> {
        if path.len() > max_depth as usize {
            Box::pin(async { None })
        } else {
            self.walk(path)
        }
    }

    /// Walks the URL tree from a string path.
    ///
    /// Only the literal empty string `""` maps to the root endpoint. Every
    /// other path — including `"/"` — is split on `'/'` and walked as a
    /// sequence of segments (empty segments are preserved).
    pub async fn walk_str(&self, path: &str) -> Option<Arc<UrlNode<C, TS>>> {
        if path.is_empty() {
            return self.root.endpoint.read().clone();
        }
        let segments: Vec<&str> = path.split('/').collect();
        self.root.clone().walk(segments.iter()).await
    }

    /// Resumable cursor over every node matching `path`, in priority
    /// order. Caller owns `segments` (split as for `walk_str`) and
    /// drains via `cursor.find_next(&segments)`. Empty `path` →
    /// empty cursor; use `walk_str("")` for the root endpoint slot.
    //
    // TODO: `futures::Stream` wrapper once a fan-out protocol needs it.
    #[av::ver(unstable, since = "0.8.1", note = "Resumable URL traversal — surface may change", date = "2026-05-25")]
    pub fn walk_cursor(&self, path: &str) -> super::node::WalkCursor<C, TS> {
        if path.is_empty() {
            return super::node::WalkCursor::empty();
        }
        super::node::WalkCursor::from_root(self.root.clone())
    }

    /// Walks the URL tree from a string path, rejecting paths deeper than `max_depth`.
    ///
    /// Depth is the total segment count after splitting on `/`, including empty
    /// segments from leading/trailing slashes or `//`. Each segment costs one
    /// recursion step regardless of content.
    pub async fn walk_str_with_limit(
        &self,
        path: &str,
        max_depth: u32,
    ) -> Option<Arc<UrlNode<C, TS>>> {
        if path.is_empty() {
            return self.root.endpoint.read().clone();
        }
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() > max_depth as usize {
            return None;
        }
        self.root.clone().walk(segments.iter()).await
    }

    /// Registers `path` under the root, returning a [`UrlRegistration`] on success.
    ///
    /// - `Ok(Root(root))` — `path` was empty; the handler was stored in the root
    ///   endpoint slot.
    /// - `Ok(Node(node))` — a child node was created or rebound.
    /// - `Err(e)` — the path pattern was invalid.
    pub(crate) fn register(
        &self,
        path: Vec<PathPattern>,
        binding: ExecutableBinding<C>,
        params: ParamsClone,
        names: StepName,
    ) -> Result<UrlRegistration<C, TS>, UrlError> {
        debug_log!("Registering URL: {:?}", path);

        if path.is_empty() {
            // The path was empty — store in the root endpoint slot.
            let new_endpoint = if let Some(existing) = self.root.endpoint.read().clone() {
                existing.rebind(binding, params, names)
            } else {
                Arc::new(UrlNode::new(
                    PathPattern::literal_path(""),
                    Children::new(),
                    binding,
                    params,
                    names,
                ))
            };
            *self.root.endpoint.write() = Some(new_endpoint);
            return Ok(UrlRegistration::Root(self.root.clone()));
        }

        if path.len() == 1 {
            let pattern = path[0].clone();
            let node = if let Some(existing) = self.root.children.find(&pattern) {
                let rebound = existing.rebind(binding, params, names);
                self.root.children.insert(rebound.clone());
                rebound
            } else {
                let child = Arc::new(UrlNode::new(
                    pattern,
                    Children::new(),
                    binding,
                    params,
                    names,
                ));
                self.root.children.insert(child.clone());
                child
            };
            return Ok(UrlRegistration::Node(node));
        }

        // Multiple segments: find or create the first-level child, then recurse.
        let first = path[0].clone();
        let first_child = if let Some(existing) = self.root.children.find(&first) {
            existing
        } else {
            let child = Arc::new(UrlNode::empty(first));
            self.root.children.insert(child.clone());
            child
        };

        first_child
            .register_relative(&path[1..], binding, params, names)
            .map(UrlRegistration::Node)
    }

    #[av::ver(deprecated, since = "0.8.0", note = "Use `register` directly. Use url::parser::parse to parse patterns before calling `register`.")]
    /// Registers a literal URL path and returns a [`UrlRegistration`].
    ///
    /// Only the empty string `""` maps to the root endpoint (`Root` variant).
    /// All other paths — including `"/"` — create tree nodes (`Node` variant).
    ///
    /// # Errors
    ///
    /// Returns [`UrlError`] if the path contains an invalid pattern.
    pub fn literal_url(
        &self,
        path: &str,
        binding: ExecutableBinding<C>,
        params: ParamsClone,
    ) -> Result<UrlRegistration<C, TS>, UrlError> {
        debug_log!("Changing url into path pattern: {}", path);
        // Only the literal empty string maps to the root endpoint.
        let path_vec: Vec<PathPattern> = if path.is_empty() {
            Vec::new()
        } else {
            path.split('/').map(PathPattern::literal_path).collect()
        };
        debug_log!("Path vector: {:?}", path_vec);
        self.register(path_vec, binding, params, StepName::default())
    }

    #[av::ver(deprecated, since = "0.8.0", note = "Use `register` directly. Use url::parser::parse to parse patterns before calling `register`.")] 
    /// Registers a URL using Hotaru pattern syntax and returns a [`UrlRegistration`].
    ///
    /// Accepts the full Hotaru pattern language (literals, `<name>`, `<type:name>`,
    /// `<regex>`, `*`, `**path`).
    ///
    /// Returns `Ok(Root(_))` only when the pattern resolves to the empty-string root.
    ///
    /// # Errors
    ///
    /// Returns [`UrlError`] if the pattern string is syntactically invalid.
    pub fn sub_url<A: AsRef<str>>(
        &self,
        path: A,
        binding: ExecutableBinding<C>,
        params: ParamsClone,
    ) -> Result<UrlRegistration<C, TS>, UrlError> {
        match parse(path.as_ref()) {
            Ok((path, names)) => self.register(path, binding, params, names.into()),
            Err(e) => Err(e.into()),
        }
    }
}

impl<C: RequestContext + Send + 'static, TS: TransportSpec> Default for UrlRoot<C, TS> {
    fn default() -> Self {
        Self::new()
    }
} 

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;

    use akari::extensions::ParamsClone;

    use crate::{
        executable::{ExecutableBinding, middleware::AsyncFinalHandler},
        protocol::{Channel, ProtocolRole},
        url::PathPattern,
    };

    use super::*;

    #[derive(Clone)]
    struct TestChannel;

    impl Channel for TestChannel {
        fn is_open(&self) -> bool { true }
        fn close(&self) {}
    }

    #[derive(Default)]
    struct TestContext;

    impl RequestContext for TestContext {
        type Request = ();
        type Response = ();
        type Error = std::io::Error;
        type Channel = TestChannel;

        fn handle_error(&mut self) {}

        fn role(&self) -> ProtocolRole {
            ProtocolRole::Server
        }

        fn inject_request(&mut self, _: Self::Request) {}
        fn into_response(self) -> Self::Response {}
    }

    fn binding_with_handler() -> ExecutableBinding<TestContext> {
        let handler: Arc<dyn AsyncFinalHandler<TestContext>> =
            Arc::new(|ctx: TestContext| async move { Ok(ctx) });
        ExecutableBinding::new().with_handler(handler)
    }

    #[tokio::test]
    async fn literal_url_registers_direct_child() {
        let root = Arc::new(UrlRoot::<TestContext>::new());
        let reg = root
            .literal_url("/users", binding_with_handler(), ParamsClone::default())
            .unwrap();
        let node = match reg {
            UrlRegistration::Node(n) => n,
            _ => panic!("direct child should return Node"),
        };

        // The leaf node is "users", not the leading empty segment.
        assert_eq!(node.path(), &PathPattern::literal_path("users"));
        assert!(node.has_handler());
        assert!(root.walk_str("/users").await.is_some());
    }

    #[tokio::test]
    async fn literal_url_creates_empty_parents_for_deep_child() {
        let root = Arc::new(UrlRoot::<TestContext>::new());
        root.literal_url("/api/users", binding_with_handler(), ParamsClone::default())
            .unwrap();

        // Intermediate nodes exist in the tree but have no handler.
        let api = root.walk_str("/api").await.unwrap();
        let users = root.walk_str("/api/users").await.unwrap();

        assert!(!api.has_handler());
        assert!(users.has_handler());
    }

    #[tokio::test]
    async fn later_registration_rebinds_existing_parent_and_preserves_children() {
        let root = Arc::new(UrlRoot::<TestContext>::new());

        root.literal_url(
            "/api/users/profile",
            binding_with_handler(),
            ParamsClone::default(),
        )
        .unwrap();

        root.literal_url("/api/users", binding_with_handler(), ParamsClone::default())
            .unwrap();

        assert!(root.walk_str("/api/users").await.is_some());
        assert!(root.walk_str("/api/users/profile").await.is_some());
    }

    #[tokio::test]
    async fn root_endpoint_is_empty_string_only() {
        let root = Arc::new(UrlRoot::<TestContext>::new());

        // Registering "" stores the handler in the root endpoint slot.
        let result = root
            .literal_url("", binding_with_handler(), ParamsClone::default())
            .unwrap();
        assert!(
            matches!(result, UrlRegistration::Root(_)),
            "empty path must return Root"
        );

        // walk_str("") reaches the root endpoint.
        let walked = root
            .walk_str("")
            .await
            .expect("walk_str('') must find the root endpoint");
        assert!(walked.has_handler());

        // "/" is a distinct tree node — not the root endpoint.
        let slash_result = root
            .literal_url("/", binding_with_handler(), ParamsClone::default())
            .unwrap();
        assert!(
            matches!(slash_result, UrlRegistration::Node(_)),
            "slash path must return Node"
        );

        // walk_str("/") reaches the slash tree node (not the root endpoint).
        assert!(root.walk_str("/").await.is_some());

        // walk_str("") still reaches the root endpoint after registering "/".
        assert!(root.walk_str("").await.is_some());
    }

    #[tokio::test]
    async fn literal_empty_segment_route_wins_over_wildcard() {
        let root = Arc::new(UrlRoot::<TestContext>::new());

        root.literal_url("/", binding_with_handler(), ParamsClone::default())
            .unwrap();
        root.sub_url("/<slug>", binding_with_handler(), ParamsClone::default())
            .unwrap();

        let endpoint = root
            .walk_str("/")
            .await
            .expect("slash path should resolve to the literal empty-segment route");

        assert_eq!(endpoint.path(), &PathPattern::literal_path(""));
        assert!(endpoint.names().index("slug").is_none());
    }

    #[tokio::test]
    async fn walk_str_with_limit_rejects_deep_paths() {
        let root = Arc::new(UrlRoot::<TestContext>::new());
        root.literal_url(
            "/api/users/profile",
            binding_with_handler(),
            ParamsClone::default(),
        )
        .unwrap();

        // "/api/users/profile" splits to ["", "api", "users", "profile"] — 4 segments.
        assert!(
            root.walk_str_with_limit("/api/users/profile", 3)
                .await
                .is_none()
        );
        assert!(
            root.walk_str_with_limit("/api/users/profile", 4)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn sub_url_registers_pattern_path() {
        let root = Arc::new(UrlRoot::<TestContext>::new());
        let reg = root
            .sub_url(
                "/users/<str:id>",
                binding_with_handler(),
                ParamsClone::default(),
            )
            .unwrap();
        let node = match reg {
            UrlRegistration::Node(n) => n,
            _ => panic!("pattern child should return Node"),
        };

        assert!(node.has_handler());
        // Parser index: [Literal("")=0, Literal("users")=1, Regex=2] — "id" maps to index 2.
        assert_eq!(node.names().index("id"), Some(2));
        assert!(root.walk_str("/users/alice").await.is_some());
    }

    #[tokio::test]
    async fn walk_cursor_yields_priority_ordered_matches() {
        let root = Arc::new(UrlRoot::<TestContext>::new());
        root.literal_url("/literal", binding_with_handler(), ParamsClone::default()).unwrap();
        root.sub_url("/<slug>", binding_with_handler(), ParamsClone::default()).unwrap();
        root.sub_url("/<**path>", binding_with_handler(), ParamsClone::default()).unwrap();

        let path = "/literal";
        let segments: Vec<&str> = path.split('/').collect();
        let mut cursor = root.walk_cursor(path);
        let mut hits: Vec<PathPattern> = Vec::new();
        while let Some(n) = cursor.find_next(&segments) {
            hits.push(n.path().clone());
        }

        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0], PathPattern::literal_path("literal"));
    }
}
