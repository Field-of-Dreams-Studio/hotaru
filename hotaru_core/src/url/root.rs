use std::{future::Future, pin::Pin, slice::Iter, sync::Arc};

use akari::extensions::ParamsClone;

use crate::{
    alias::PRwLock,
    connection::TransportSpec,
    debug_log,
    executable::ExecutableBinding,
    protocol::RequestContext,
    url::{PathPattern, UrlError},
};

use super::{node::UrlNode, parser::parse};

/// Root wrapper for a URL tree.
pub struct UrlRoot<C: RequestContext, TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    root: PRwLock<Arc<UrlNode<C, TS>>>,
}

impl<C: RequestContext + Send + 'static, TS: TransportSpec> UrlRoot<C, TS> {
    /// Creates a new URL root from the starting node.
    pub fn new(root: Arc<UrlNode<C, TS>>) -> Self {
        Self {
            root: PRwLock::new(root),
        }
    }

    /// Returns the starting node of this URL tree.
    pub fn root(&self) -> Arc<UrlNode<C, TS>> {
        self.root.read().clone()
    }

    /// Walks the URL tree using path segments.
    pub fn walk<'a>(
        &self,
        path: Iter<'a, &str>,
    ) -> Pin<Box<dyn Future<Output = Option<Arc<UrlNode<C, TS>>>> + Send + 'a>> {
        self.root().walk(path)
    }

    /// Walks the URL tree using path segments, rejecting paths deeper than `max_depth`.
    pub fn walk_with_limit<'a>(
        &self,
        path: Iter<'a, &str>,
        max_depth: u32,
    ) -> Pin<Box<dyn Future<Output = Option<Arc<UrlNode<C, TS>>>> + Send + 'a>> {
        if path.len() > max_depth as usize {
            Box::pin(async { None })
        } else {
            self.walk(path)
        }
    }

    /// Walks the URL tree from a string path.
    pub async fn walk_str(&self, path: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.root().walk_str(path).await
    }

    /// Walks the URL tree from a string path, rejecting paths deeper than `max_depth`.
    pub async fn walk_str_with_limit(&self, path: &str, max_depth: u32) -> Option<Arc<UrlNode<C, TS>>> {
        self.root().walk_str_with_limit(path, max_depth).await
    }

    fn register(
        self: Arc<Self>,
        mut path: Vec<PathPattern>,
        binding: ExecutableBinding<C>,
        params: ParamsClone,
        names: super::node::StepName,
    ) -> Result<Arc<UrlNode<C, TS>>, UrlError> {
        debug_log!("Registering URL: {:?}", path);
        let root = self.root();
        let root_path = root.path().clone();

        if !matches!(&root_path, PathPattern::Literal(path) if path.is_empty() || path == "/") {
            if path.first() == Some(&root_path) {
                path.remove(0);
            }
        }

        if path.is_empty() {
            let rebound = root.rebind(binding, params, names);
            *self.root.write() = rebound.clone();
            return Ok(rebound);
        }

        root.register_relative(&path, binding, params, names)
    }

    /// Register a child URL with a function.
    pub fn literal_url(
        self: Arc<Self>,
        path: &str,
        binding: ExecutableBinding<C>, 
        params: ParamsClone,
    ) -> Result<Arc<UrlNode<C, TS>>, UrlError> {
        debug_log!("Changing url into path pattern: {}", path);
        // Remove the first slash if exist
        let path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        // Use register, convert the path to a Vec<PathPattern>
        let path_vec: Vec<PathPattern> = if path.is_empty() {
            Vec::new()
        } else {
            path.split('/').map(|s| PathPattern::literal_path(s)).collect()
        };

        debug_log!("Path vector: {:?}", path_vec);
        self.register(path_vec, binding, params, super::node::StepName::default())
    }

    // Using Hotaru pattern to register URL
    pub fn sub_url<A: AsRef<str>>(
        self: &Arc<Self>,
        path: A,
        binding: ExecutableBinding<C>,
        params: ParamsClone,
    ) -> Result<Arc<UrlNode<C, TS>>, UrlError> {
        match parse(path.as_ref()) {
            Ok((path, names)) => self.clone().register(path, binding, params, names.into()),
            Err(e) => Err(UrlError::ParseError(e.to_string())),
        }
    }

}

impl<C: RequestContext + Send + 'static, TS: TransportSpec> Default for UrlRoot<C, TS> {
    fn default() -> Self {
        Self::new(Arc::new(UrlNode::empty(PathPattern::literal_path(""))))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        executable::{middleware::AsyncFinalHandler, ExecutableBinding},
        protocol::ProtocolRole,
        url::{Children, PathPattern, StepName},
    };

    use super::*;

    struct TestContext;

    impl RequestContext for TestContext {
        type Request = ();
        type Response = ();

        fn handle_error(&mut self) {}

        fn role(&self) -> ProtocolRole {
            ProtocolRole::Server
        }
    }

    fn root_node(path: PathPattern) -> Arc<UrlNode<TestContext>> {
        Arc::new(UrlNode::new(
            path,
            Children::new(),
            ExecutableBinding::new(),
            ParamsClone::default(),
            StepName::default(),
        ))
    }

    fn binding_with_handler() -> ExecutableBinding<TestContext> {
        let handler: Arc<dyn AsyncFinalHandler<TestContext>> =
            Arc::new(|ctx: TestContext| async move { ctx });
        ExecutableBinding::new().with_handler(handler)
    }

    #[test]
    fn root_new_returns_starting_node() {
        let node = root_node(PathPattern::literal_path(""));
        let root = UrlRoot::new(node.clone());
        assert!(Arc::ptr_eq(&root.root(), &node));
    }

    #[tokio::test]
    async fn literal_url_registers_direct_child() {
        let root = Arc::new(UrlRoot::new(root_node(PathPattern::literal_path(""))));
        let node = root
            .clone()
            .literal_url("/users", binding_with_handler(), ParamsClone::default())
            .unwrap();

        assert_eq!(node.path(), &PathPattern::literal_path("users"));
        assert!(node.has_handler());
        assert!(root.walk_str("/users").await.is_some());
    }

    #[tokio::test]
    async fn literal_url_creates_empty_parents_for_deep_child() {
        let root = Arc::new(UrlRoot::new(root_node(PathPattern::literal_path(""))));
        root.clone()
            .literal_url("/api/users", binding_with_handler(), ParamsClone::default())
            .unwrap();

        let api = root.walk_str("/api").await.unwrap();
        let users = root.walk_str("/api/users").await.unwrap();

        assert!(!api.has_handler());
        assert!(users.has_handler());
    }

    #[tokio::test]
    async fn later_registration_rebinds_existing_parent_and_preserves_children() {
        let root = Arc::new(UrlRoot::new(root_node(PathPattern::literal_path(""))));

        root.clone()
            .literal_url(
                "/api/users/profile",
                binding_with_handler(),
                ParamsClone::default(),
            )
            .unwrap();

        let users = root
            .clone()
            .literal_url("/api/users", binding_with_handler(), ParamsClone::default())
            .unwrap();

        assert!(users.has_handler());
        assert!(root.walk_str("/api/users/profile").await.is_some());
    }

    #[tokio::test]
    async fn literal_url_can_rebind_root_when_path_is_empty() {
        let root = Arc::new(UrlRoot::new(root_node(PathPattern::literal_path(""))));
        let rebound = root
            .clone()
            .literal_url("/", binding_with_handler(), ParamsClone::default())
            .unwrap();

        assert!(rebound.has_handler());
        assert!(root.root().has_handler());
        assert!(root.walk_str("/").await.is_some());
    }

    #[tokio::test]
    async fn register_strips_root_prefix_when_it_matches_root_node() {
        let root = Arc::new(UrlRoot::new(root_node(PathPattern::literal_path("api"))));
        let node = root
            .clone()
            .literal_url("/api/users", binding_with_handler(), ParamsClone::default())
            .unwrap();

        assert_eq!(node.path(), &PathPattern::literal_path("users"));
        assert!(root.root().find_child(&PathPattern::literal_path("users")).is_some());
        assert!(root.walk_str("/users").await.is_some());
    }

    #[tokio::test]
    async fn walk_str_with_limit_rejects_deep_paths() {
        let root = Arc::new(UrlRoot::new(root_node(PathPattern::literal_path(""))));
        root.clone()
            .literal_url(
                "/api/users/profile",
                binding_with_handler(),
                ParamsClone::default(),
            )
            .unwrap();

        assert!(root.walk_str_with_limit("/api/users/profile", 2).await.is_none());
        assert!(root.walk_str_with_limit("/api/users/profile", 3).await.is_some());
    }

    #[tokio::test]
    async fn sub_url_registers_pattern_path() {
        let root = Arc::new(UrlRoot::new(root_node(PathPattern::literal_path(""))));
        let node = root
            .sub_url("/users/<str:id>", binding_with_handler(), ParamsClone::default())
            .unwrap();

        assert!(node.has_handler());
        assert_eq!(node.names().index("id"), Some(1));
        assert!(root.walk_str("/users/alice").await.is_some());
    }
}
