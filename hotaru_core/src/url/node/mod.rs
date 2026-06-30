use alloc::sync::Arc;
use core::slice::Iter;

use crate::{
    connection::TransportSpec,
    executable::{ExecutableBinding, ExecutionChain},
    extensions::{ParamValue, ParamsClone},
    marker::MaybeSendBoxFuture,
    protocol::RequestContext,
    url::PathPattern,
};

mod children;
mod cursor;
mod partial;
mod stepname;

pub use self::children::{Children, ChildrenInner, LiteralChild, RegexChild};
pub use self::cursor::{FrameNode, WalkCursor, WalkFrame};
pub use self::partial::PartialState;
pub use self::stepname::StepName;

/// Represents a URL in the application.
/// This struct holds the various components of a URL, including its path, query parameters, and more.
pub struct UrlNode<C: RequestContext, TS: TransportSpec> {
    // The last segment of the URL path
    path: PathPattern,

    // The child segments of the URL path
    children: Children<C, TS>,

    // Immutable executable definition attached to this route node.
    binding: Arc<ExecutableBinding<C>>,

    // Compiled execution payload used on the hot path.
    chain: Option<ExecutionChain<C>>,

    // The static config of the URL.
    // If a route-level value must remain mutable at runtime, store
    // `Arc<PRwLock<T>>` inside these params rather than locking the whole set.
    params: ParamsClone,

    // The step names of the URL path
    names: StepName,
}

impl<C: RequestContext + Send + 'static, TS: TransportSpec> UrlNode<C, TS> {
    pub fn new(
        path: PathPattern,
        children: Children<C, TS>,
        binding: ExecutableBinding<C>,
        params: ParamsClone,
        names: StepName,
    ) -> Self {
        let chain = binding.compile();
        Self {
            path,
            children,
            binding: Arc::new(binding),
            chain,
            params,
            names,
        }
    }

    fn from_parts(
        path: PathPattern,
        children: Children<C, TS>,
        binding: Arc<ExecutableBinding<C>>,
        chain: Option<ExecutionChain<C>>,
        params: ParamsClone,
        names: StepName,
    ) -> Self {
        Self {
            path,
            children,
            binding,
            chain,
            params,
            names,
        }
    }

    pub fn path(&self) -> &PathPattern {
        &self.path
    }

    pub fn empty(path: PathPattern) -> Self {
        Self {
            path,
            children: Children::new(),
            binding: Arc::new(ExecutableBinding::new()),
            chain: None,
            params: ParamsClone::default(),
            names: StepName::default(),
        }
    }

    pub fn dangling_url() -> Arc<Self> {
        Arc::new(Self::empty(PathPattern::Any))
    }

    pub fn children(&self) -> &Children<C, TS> {
        &self.children
    }

    pub fn remove(&self, pattern: &PathPattern) -> Option<Arc<UrlNode<C, TS>>> {
        self.children.remove(pattern)
    }

    pub fn find_child(&self, pattern: &PathPattern) -> Option<Arc<UrlNode<C, TS>>> {
        self.children.find(pattern)
    }

    pub fn insert_child(&self, child: Arc<UrlNode<C, TS>>) -> Arc<UrlNode<C, TS>> {
        self.children.insert(child.clone());
        child
    }

    pub fn get_child_or_create(self: &Arc<Self>, pattern: PathPattern) -> Arc<UrlNode<C, TS>> {
        if let Some(existing) = self.find_child(&pattern) {
            existing
        } else {
            let child = Arc::new(Self::empty(pattern));
            self.insert_child(child.clone())
        }
    }

    pub fn binding(&self) -> Arc<ExecutableBinding<C>> {
        self.binding.clone()
    }

    pub fn chain(&self) -> Option<ExecutionChain<C>> {
        self.chain.clone()
    }

    pub fn has_handler(&self) -> bool {
        self.chain.is_some()
    }

    pub fn params(&self) -> &ParamsClone {
        &self.params
    }

    pub fn names(&self) -> &StepName {
        &self.names
    }

    pub fn match_seg_name_with_index<A: AsRef<str>>(&self, name: A) -> Option<usize> {
        self.names.index(name)
    }

    /// Walks the URL tree using the provided path segments.
    ///
    /// Returns the matched node if traversal succeeds, or `None` if no route matches.
    ///
    /// Matching order for each step is:
    /// 1. literal
    /// 2. regex
    /// 3. single-segment wildcard (`*`)
    /// 4. catch-all wildcard (`**`) as fallback
    ///
    /// `AnyPath` is intentionally the weakest match. It is only used after the
    /// stronger candidates fail, including cases where a more specific branch
    /// matched earlier but could not complete deeper in the tree.
    ///
    /// Example:
    /// - if `/<**path>`, `/a`, and `/a/b/c` all exist:
    ///   - `/a` matches `/a`
    ///   - `/a/b/c` matches `/a/b/c`
    ///   - `/a/b/d` falls back to `/<**path>`
    ///
    /// # Security Note
    ///
    /// This traversal does not impose an explicit user-path depth limit because the
    /// route tree is fixed by application code. User input can only traverse
    /// existing nodes; it cannot create new ones. Extra path segments simply fail to
    /// match and return `None`.
    ///
    /// If a future design introduces dynamic route creation or cyclic node graphs,
    /// depth validation should be revisited at that layer.
    pub fn walk<'a>(
        self: Arc<Self>,
        mut path: Iter<'a, &str>,
        mut state: PartialState,
    ) -> MaybeSendBoxFuture<'a, Option<Arc<Self>>> {
        let this_segment = match path.next() {
            Some(segment) => *segment,
            None => return Box::pin(async move { Some(self) }),
        };

        Box::pin(async move {
            while !state.is_end() {
                let (matched_child, next_state) = self.children.match_step(this_segment, state);
                state = next_state;

                let Some(child) = matched_child else {
                    continue;
                };

                if path.len() >= 1 && !child.path().is_any_path() {
                    if let Some(result) = child
                        .clone()
                        .walk(path.clone(), PartialState::NotStart)
                        .await
                    {
                        return Some(result);
                    }
                } else {
                    return Some(child);
                }
            }

            None
        })
    }

    pub async fn walk_str(self: Arc<Self>, path: &str) -> Option<Arc<Self>> {
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        self.walk(segments.iter(), PartialState::NotStart).await
    }

    /// Walks the URL tree from a string path, rejecting paths deeper than `max_depth`.
    pub async fn walk_str_with_limit(
        self: Arc<Self>,
        path: &str,
        max_depth: u32,
    ) -> Option<Arc<Self>> {
        let segments: Vec<&str> = path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.len() > max_depth as usize {
            return None;
        }
        self.walk(segments.iter(), PartialState::NotStart).await
    }

    /// Retrieves a cloned value of type `T` from the URL's parameter storage.
    /// Returns `Some(T)` if the parameter exists and matches the type, `None` otherwise.
    pub fn get_params<T: ParamValue + Clone + 'static>(&self) -> Option<T> {
        self.params.get::<T>().cloned()
    }

    /// Stores a value in the URL's parameter storage, overwriting any existing value.
    pub fn set_params<T: ParamValue + 'static>(&mut self, value: T) {
        self.params.set(value);
    }

    pub fn rebind(
        self: &Arc<Self>,
        binding: ExecutableBinding<C>,
        params: ParamsClone,
        names: StepName,
    ) -> Arc<Self> {
        let mut merged_params = self.params.clone();
        merged_params.merge(&params);
        let names = if names.is_empty() {
            self.names.clone()
        } else {
            names
        };

        Arc::new(Self::from_parts(
            self.path.clone(),
            self.children.clone(),
            Arc::new(binding.clone()),
            binding.compile(),
            merged_params,
            names,
        ))
    }

    pub fn register_relative(
        self: Arc<Self>,
        path: &[PathPattern],
        binding: ExecutableBinding<C>,
        params: ParamsClone,
        names: StepName,
    ) -> Result<Arc<Self>, crate::url::UrlError> {
        if path.is_empty() {
            return Err(crate::url::UrlError::InvalidPath(
                "register_relative requires at least one segment".to_string(),
            ));
        }

        if path.len() == 1 {
            let pattern = path[0].clone();
            if let Some(existing) = self.find_child(&pattern) {
                let rebound = existing.rebind(binding, params, names);
                self.insert_child(rebound.clone());
                Ok(rebound)
            } else {
                let child = Arc::new(Self::new(pattern, Children::new(), binding, params, names));
                self.insert_child(child.clone());
                Ok(child)
            }
        } else {
            let child = self.get_child_or_create(path[0].clone());
            child.register_relative(&path[1..], binding, params, names)
        }
    }

    pub async fn run(&self, mut ctx: C) -> Result<C, <C as RequestContext>::Error> {
        if let Some(chain) = &self.chain {
            chain.run(ctx).await
        } else {
            ctx.handle_error();
            Ok(ctx)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;

    use crate::{
        connection::test_support::TestTransport,
        protocol::{Channel, ProtocolRole},
        url::PathPattern,
    };

    use super::*;

    #[derive(Clone)]
    struct TestChannel;

    impl Channel for TestChannel {
        fn is_open(&self) -> bool {
            true
        }
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

    type TestNode = UrlNode<TestContext, TestTransport>;

    fn empty_node(pattern: PathPattern) -> Arc<TestNode> {
        Arc::new(TestNode::empty(pattern))
    }

    fn segments<'a>(path: &'a [&'a str]) -> Vec<&'a str> {
        path.to_vec()
    }

    #[tokio::test]
    async fn walk_matches_direct_literal_child() {
        let root = empty_node(PathPattern::literal_path("files"));
        let a = empty_node(PathPattern::literal_path("a"));
        root.insert_child(a.clone());

        let path = segments(&["a"]);
        let matched = root
            .walk(path.iter(), PartialState::NotStart)
            .await
            .expect("literal child should match");

        assert!(Arc::ptr_eq(&matched, &a));
    }

    #[tokio::test]
    async fn walk_matches_nested_literal_child() {
        let root = empty_node(PathPattern::literal_path("files"));
        let a = empty_node(PathPattern::literal_path("a"));
        let b = empty_node(PathPattern::literal_path("b"));
        root.insert_child(a.clone());
        a.insert_child(b.clone());

        let path = segments(&["a", "b"]);
        let matched = root
            .walk(path.iter(), PartialState::NotStart)
            .await
            .expect("nested literal child should match");

        assert!(Arc::ptr_eq(&matched, &b));
    }

    #[tokio::test]
    async fn walk_any_path_matches_one_remaining_segment() {
        let root = empty_node(PathPattern::literal_path("files"));
        let rest = empty_node(PathPattern::AnyPath);
        root.insert_child(rest.clone());

        let path = segments(&["a"]);
        let matched = root
            .walk(path.iter(), PartialState::NotStart)
            .await
            .expect("AnyPath currently matches one segment");

        assert!(Arc::ptr_eq(&matched, &rest));
    }

    #[tokio::test]
    async fn walk_any_path_should_catch_multiple_remaining_segments() {
        let root = empty_node(PathPattern::literal_path("files"));
        let rest = empty_node(PathPattern::AnyPath);
        root.insert_child(rest.clone());

        let path = segments(&["a", "b", "c"]);
        let matched = root
            .walk(path.iter(), PartialState::NotStart)
            .await
            .expect("AnyPath should match /files/a/b/c");

        assert!(Arc::ptr_eq(&matched, &rest));
    }
}
