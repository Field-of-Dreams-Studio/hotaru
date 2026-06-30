use akari::hash::HashMap;
use alloc::sync::Arc;
use core::marker::PhantomData;

use crate::{
    alias::PRwLock,
    connection::TransportSpec,
    protocol::RequestContext,
    url::{PathPattern, RegexSegment},
};

use super::{PartialState, UrlNode};

/// Child-node storage wrapper used by a URL node.
///
/// The lock is expected to live at the `Children` layer so all child-cache
/// updates remain atomic from the caller's perspective.
pub struct Children<C: RequestContext, TS: TransportSpec> {
    inner: PRwLock<ChildrenInner<C, TS>>,
    _ts: PhantomData<TS>,
}

/// Inner child-node caches grouped by match strategy.
///
/// Lookup order is intended to be:
/// 1. exact literal match
/// 2. regex scan
/// 3. single-segment wildcard
/// 4. catch-all wildcard
pub struct ChildrenInner<C: RequestContext, TS: TransportSpec> {
    /// Exact-match children keyed by literal segment.
    literals: LiteralChild<C, TS>,
    /// Regex children evaluated after exact literal lookup.
    regex: Vec<RegexChild<C, TS>>,
    /// Optional single-segment wildcard child.
    any: Option<Arc<UrlNode<C, TS>>>,
    /// Optional catch-all wildcard child.
    any_path: Option<Arc<UrlNode<C, TS>>>,
    _ts: PhantomData<TS>,
}

/// Exact-match child cache for literal path segments.
pub struct LiteralChild<C: RequestContext, TS: TransportSpec> {
    inner: HashMap<String, Arc<UrlNode<C, TS>>>,
    _ts: PhantomData<TS>,
}

/// Regex child entry kept in insertion order for fallback scanning.
///
/// Holds the compiled `RegexSegment` (shared with the originating
/// `PathPattern::Regex` via `Arc`), so matching against incoming segments
/// never recompiles the regex.
pub struct RegexChild<C: RequestContext, TS: TransportSpec> {
    seg: RegexSegment,
    node: Arc<UrlNode<C, TS>>,
    _ts: PhantomData<TS>,
}

impl<C: RequestContext, TS: TransportSpec> Children<C, TS> {
    /// Creates an empty children cache wrapper.
    pub fn new() -> Self {
        Self {
            inner: PRwLock::new(ChildrenInner::new()),
            _ts: PhantomData,
        }
    }

    /// Returns whether there are no children registered.
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Returns the total number of registered children.
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Inserts a child into the appropriate cache bucket.
    pub fn insert(&self, child: Arc<UrlNode<C, TS>>) {
        self.inner.write().insert(child);
    }

    /// Removes and returns a child matching the exact path pattern.
    pub fn remove(&self, pattern: &PathPattern) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.write().remove(pattern)
    }

    /// Finds a child by exact registered path pattern.
    pub fn find(&self, pattern: &PathPattern) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.read().find(pattern)
    }

    /// Matches a runtime path segment using cache order.
    pub fn match_segment(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.read().match_segment(segment)
    }

    /// Matches a literal child by exact segment.
    pub fn match_literal(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.read().match_literal(segment)
    }

    /// Matches the first regex child that accepts the segment.
    pub fn match_regex(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.read().match_regex(segment)
    }

    /// Matches the first regex child that accepts the segment at or after `idx`.
    pub fn match_regex_with_idx(
        &self,
        segment: &str,
        idx: usize,
    ) -> Option<(Arc<UrlNode<C, TS>>, usize)> {
        self.inner.read().match_regex_with_idx(segment, idx)
    }

    /// Matches the single-segment wildcard child.
    pub fn match_any(&self) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.read().match_any()
    }

    /// Matches the catch-all wildcard child.
    pub fn match_any_path(&self) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.read().match_any_path()
    }

    /// Matches one segment using literal, regex, then wildcard order.
    pub fn match_one_segment(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.read().match_one_segment(segment)
    }

    /// Matches one step and returns the next candidate plus resume state.
    ///
    /// `Any` accepts any single segment, including the empty string `""`.
    /// The effective priority is `Literal > Regex > Any > AnyPath`, so an
    /// explicit literal empty-segment route (for example `"/"`) always wins
    /// over a wildcard route such as `"/<slug>"`.
    pub fn match_step(
        &self,
        segment: &str,
        state: PartialState,
    ) -> (Option<Arc<UrlNode<C, TS>>>, PartialState) {
        self.inner.read().match_step(segment, state)
    }

    /// Returns all child nodes for traversal or debug use.
    pub fn all_nodes(&self) -> Vec<Arc<UrlNode<C, TS>>> {
        self.inner.read().all_nodes()
    }

    /// Formats the cached children for debug display.
    pub fn display_string(&self) -> String {
        self.inner.read().display_string()
    }
}

impl<C: RequestContext, TS: TransportSpec> Clone for Children<C, TS> {
    fn clone(&self) -> Self {
        Self {
            inner: PRwLock::new(self.inner.read().clone()),
            _ts: PhantomData,
        }
    }
}

impl<C: RequestContext, TS: TransportSpec> ChildrenInner<C, TS> {
    /// Creates an empty inner children cache.
    pub fn new() -> Self {
        Self {
            literals: LiteralChild::new(),
            regex: Vec::new(),
            any: None,
            any_path: None,
            _ts: PhantomData,
        }
    }

    /// Returns whether the cache has no children.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total cached child count.
    pub fn len(&self) -> usize {
        self.literals.len()
            + self.regex.len()
            + usize::from(self.any.is_some())
            + usize::from(self.any_path.is_some())
    }

    /// Inserts a child into the correct cache entry.
    pub fn insert(&mut self, child: Arc<UrlNode<C, TS>>) {
        match &child.path {
            PathPattern::Literal(segment) => self.literals.insert(segment.clone(), child),
            PathPattern::Regex(seg) => {
                self.regex.retain(|entry| entry.pattern() != seg.src());
                self.regex.push(RegexChild::new(seg.clone(), child));
            }
            PathPattern::Any => self.any = Some(child),
            PathPattern::AnyPath => self.any_path = Some(child),
        }
    }

    /// Removes and returns a child by exact path pattern identity.
    pub fn remove(&mut self, pattern: &PathPattern) -> Option<Arc<UrlNode<C, TS>>> {
        match pattern {
            PathPattern::Literal(segment) => self.literals.remove(segment),
            PathPattern::Regex(seg) => {
                if let Some(pos) = self
                    .regex
                    .iter()
                    .position(|entry| entry.pattern() == seg.src())
                {
                    Some(self.regex.remove(pos).node())
                } else {
                    None
                }
            }
            PathPattern::Any => self.any.take(),
            PathPattern::AnyPath => self.any_path.take(),
        }
    }

    /// Finds a child by exact path pattern identity.
    pub fn find(&self, pattern: &PathPattern) -> Option<Arc<UrlNode<C, TS>>> {
        match pattern {
            PathPattern::Literal(segment) => self.literals.get(segment),
            PathPattern::Regex(seg) => self
                .regex
                .iter()
                .find(|entry| entry.pattern() == seg.src())
                .map(|entry| entry.node()),
            PathPattern::Any => self.any.clone(),
            PathPattern::AnyPath => self.any_path.clone(),
        }
    }

    /// Matches a literal child by exact segment.
    pub fn match_literal(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.literals.get(segment)
    }

    /// Matches the first regex child that accepts the segment.
    pub fn match_regex(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.regex
            .iter()
            .find(|entry| entry.matches(segment))
            .map(|entry| entry.node())
    }

    /// Matches the first regex child that accepts the segment at or after `idx`.
    pub fn match_regex_with_idx(
        &self,
        segment: &str,
        idx: usize,
    ) -> Option<(Arc<UrlNode<C, TS>>, usize)> {
        self.regex
            .iter()
            .enumerate()
            .skip(idx)
            .find(|(_, entry)| entry.matches(segment))
            .map(|(idx, entry)| (entry.node(), idx))
    }

    /// Matches the single-segment wildcard child.
    pub fn match_any(&self) -> Option<Arc<UrlNode<C, TS>>> {
        self.any.clone()
    }

    /// Matches the catch-all wildcard child.
    pub fn match_any_path(&self) -> Option<Arc<UrlNode<C, TS>>> {
        self.any_path.clone()
    }

    /// Matches one segment using literal, regex, then wildcard order.
    pub fn match_one_segment(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.match_literal(segment)
            .or_else(|| self.match_regex(segment))
            .or_else(|| self.match_any())
    }

    /// Matches one step and returns the next candidate plus resume state.
    pub fn match_step(
        &self,
        segment: &str,
        state: PartialState,
    ) -> (Option<Arc<UrlNode<C, TS>>>, PartialState) {
        match state {
            PartialState::NotStart => {
                if let Some(node) = self.match_literal(segment) {
                    return (Some(node), PartialState::Lit);
                }
                if let Some((node, idx)) = self.match_regex_with_idx(segment, 0) {
                    return (Some(node), PartialState::Reg(idx));
                }
                if let Some(node) = self.match_any() {
                    return (Some(node), PartialState::Any);
                }
                if let Some(node) = self.match_any_path() {
                    return (Some(node), PartialState::AnyPath);
                }
                (None, PartialState::End)
            }
            PartialState::Lit => {
                if let Some((node, idx)) = self.match_regex_with_idx(segment, 0) {
                    return (Some(node), PartialState::Reg(idx));
                }
                if let Some(node) = self.match_any() {
                    return (Some(node), PartialState::Any);
                }
                if let Some(node) = self.match_any_path() {
                    return (Some(node), PartialState::AnyPath);
                }
                (None, PartialState::End)
            }
            PartialState::Reg(idx) => {
                if let Some((node, next_idx)) = self.match_regex_with_idx(segment, idx + 1) {
                    return (Some(node), PartialState::Reg(next_idx));
                }
                if let Some(node) = self.match_any() {
                    return (Some(node), PartialState::Any);
                }
                if let Some(node) = self.match_any_path() {
                    return (Some(node), PartialState::AnyPath);
                }
                (None, PartialState::End)
            }
            PartialState::Any => {
                if let Some(node) = self.match_any_path() {
                    return (Some(node), PartialState::AnyPath);
                }
                (None, PartialState::End)
            }
            PartialState::AnyPath | PartialState::End => (None, PartialState::End),
        }
    }

    /// Matches a runtime path segment across all cache buckets.
    pub fn match_segment(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.match_step(segment, PartialState::NotStart).0
    }

    /// Returns every stored child node.
    pub fn all_nodes(&self) -> Vec<Arc<UrlNode<C, TS>>> {
        let mut nodes = self.literals.all_nodes();
        nodes.extend(self.regex.iter().map(|entry| entry.node()));
        if let Some(node) = &self.any {
            nodes.push(node.clone());
        }
        if let Some(node) = &self.any_path {
            nodes.push(node.clone());
        }
        nodes
    }

    /// Formats the cache contents for debug output.
    pub fn display_string(&self) -> String {
        let mut result = String::new();
        for child in self.all_nodes() {
            result.push_str(&format!("{}\n", child.path));
        }
        result
    }
}

impl<C: RequestContext, TS: TransportSpec> Clone for ChildrenInner<C, TS> {
    fn clone(&self) -> Self {
        Self {
            literals: self.literals.clone(),
            regex: self.regex.clone(),
            any: self.any.clone(),
            any_path: self.any_path.clone(),
            _ts: PhantomData,
        }
    }
}

impl<C: RequestContext, TS: TransportSpec> LiteralChild<C, TS> {
    /// Creates an empty literal child cache.
    pub fn new() -> Self {
        Self {
            inner: HashMap::default(),
            _ts: PhantomData,
        }
    }

    /// Returns whether there are no literal children.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of literal children.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Inserts or replaces a literal child by exact segment.
    pub fn insert<T: Into<String>>(&mut self, segment: T, node: Arc<UrlNode<C, TS>>) {
        self.inner.insert(segment.into(), node);
    }

    /// Removes a literal child by exact segment.
    pub fn remove(&mut self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.remove(segment)
    }

    /// Finds a literal child by exact segment.
    pub fn get(&self, segment: &str) -> Option<Arc<UrlNode<C, TS>>> {
        self.inner.get(segment).cloned()
    }

    /// Returns every literal child node.
    pub fn all_nodes(&self) -> Vec<Arc<UrlNode<C, TS>>> {
        let mut keys: Vec<_> = self.inner.keys().cloned().collect();
        keys.sort();
        keys.into_iter()
            .filter_map(|key| self.inner.get(&key).cloned())
            .collect()
    }
}

impl<C: RequestContext, TS: TransportSpec> Clone for LiteralChild<C, TS> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _ts: PhantomData,
        }
    }
}

impl<C: RequestContext, TS: TransportSpec> RegexChild<C, TS> {
    /// Creates a regex child entry from a compiled `RegexSegment`.
    pub fn new(seg: RegexSegment, node: Arc<UrlNode<C, TS>>) -> Self {
        Self {
            seg,
            node,
            _ts: PhantomData,
        }
    }

    /// Returns whether this regex child matches the segment. Reuses the
    /// cached compiled regex; no recompilation per call.
    pub fn matches(&self, segment: &str) -> bool {
        self.seg.is_match(segment)
    }

    /// Returns the stored regex pattern source.
    pub fn pattern(&self) -> &str {
        self.seg.src()
    }

    /// Returns the node attached to this regex child.
    pub fn node(&self) -> Arc<UrlNode<C, TS>> {
        self.node.clone()
    }
}

impl<C: RequestContext, TS: TransportSpec> Clone for RegexChild<C, TS> {
    fn clone(&self) -> Self {
        Self {
            seg: self.seg.clone(),
            node: self.node.clone(),
            _ts: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use akari::hash::HashMap;
    use alloc::sync::Arc;

    use crate::{
        connection::tcp::TcpTransport,
        executable::ExecutableBinding,
        extensions::ParamsClone,
        protocol::{Channel, ProtocolRole, RequestContext},
        url::{PathPattern, StepName},
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

    fn test_node(pattern: PathPattern) -> Arc<UrlNode<TestContext, TcpTransport>> {
        let binding = ExecutableBinding::new();
        Arc::new(UrlNode {
            path: pattern,
            children: Children::new(),
            chain: binding.compile(),
            binding: Arc::new(binding),
            params: ParamsClone::default(),
            names: StepName {
                inner: HashMap::default(),
            },
        })
    }

    #[test]
    fn children_new_is_empty() {
        let children = Children::<TestContext, TcpTransport>::new();
        assert!(children.is_empty());
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn children_insert_literal_and_find() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let node = test_node(PathPattern::Literal("home".into()));
        children.insert(node.clone());
        let found = children.find(&PathPattern::Literal("home".into())).unwrap();
        assert!(Arc::ptr_eq(&found, &node));
    }

    #[test]
    fn children_insert_literal_replaces_existing() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let first = test_node(PathPattern::Literal("home".into()));
        let second = test_node(PathPattern::Literal("home".into()));
        children.insert(first);
        children.insert(second.clone());
        assert_eq!(children.len(), 1);
        let found = children.find(&PathPattern::Literal("home".into())).unwrap();
        assert!(Arc::ptr_eq(&found, &second));
    }

    #[test]
    fn children_insert_regex_and_find() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let node = test_node(PathPattern::regex_path("^user[0-9]+$"));
        children.insert(node.clone());
        let found = children
            .find(&PathPattern::regex_path("^user[0-9]+$"))
            .unwrap();
        assert!(Arc::ptr_eq(&found, &node));
    }

    #[test]
    fn children_insert_any_and_find() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let node = test_node(PathPattern::Any);
        children.insert(node.clone());
        let found = children.find(&PathPattern::Any).unwrap();
        assert!(Arc::ptr_eq(&found, &node));
    }

    #[test]
    fn children_insert_any_path_and_find() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let node = test_node(PathPattern::AnyPath);
        children.insert(node.clone());
        let found = children.find(&PathPattern::AnyPath).unwrap();
        assert!(Arc::ptr_eq(&found, &node));
    }

    #[test]
    fn children_match_prefers_literal_over_others() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let literal = test_node(PathPattern::Literal("user42".into()));
        let regex = test_node(PathPattern::regex_path("^user[0-9]+$"));
        let any = test_node(PathPattern::Any);
        let any_path = test_node(PathPattern::AnyPath);
        children.insert(regex);
        children.insert(any);
        children.insert(any_path);
        children.insert(literal.clone());

        let found = children.match_segment("user42").unwrap();
        assert!(Arc::ptr_eq(&found, &literal));
    }

    #[test]
    fn children_match_uses_regex_when_no_literal() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let regex = test_node(PathPattern::regex_path("^user[0-9]+$"));
        children.insert(regex.clone());

        let found = children.match_segment("user42").unwrap();
        assert!(Arc::ptr_eq(&found, &regex));
    }

    #[test]
    fn children_match_uses_any_when_no_literal_or_regex() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let any = test_node(PathPattern::Any);
        children.insert(any.clone());

        let found = children.match_segment("whatever").unwrap();
        assert!(Arc::ptr_eq(&found, &any));
    }

    #[test]
    fn children_match_uses_any_path_last() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let any_path = test_node(PathPattern::AnyPath);
        children.insert(any_path.clone());

        let found = children.match_segment("rest/of/path").unwrap();
        assert!(Arc::ptr_eq(&found, &any_path));
    }

    #[test]
    fn children_remove_literal() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let node = test_node(PathPattern::Literal("home".into()));
        children.insert(node.clone());
        let removed = children
            .remove(&PathPattern::Literal("home".into()))
            .unwrap();
        assert!(Arc::ptr_eq(&removed, &node));
        assert!(
            children
                .find(&PathPattern::Literal("home".into()))
                .is_none()
        );
    }

    #[test]
    fn children_remove_regex() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let node = test_node(PathPattern::regex_path("^user[0-9]+$"));
        children.insert(node.clone());
        let removed = children
            .remove(&PathPattern::regex_path("^user[0-9]+$"))
            .unwrap();
        assert!(Arc::ptr_eq(&removed, &node));
        assert!(
            children
                .find(&PathPattern::regex_path("^user[0-9]+$"))
                .is_none()
        );
    }

    #[test]
    fn children_remove_missing_returns_none() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        assert!(
            children
                .remove(&PathPattern::Literal("missing".into()))
                .is_none()
        );
    }

    #[test]
    fn children_all_nodes_returns_every_bucket() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        children.insert(test_node(PathPattern::Literal("a".into())));
        children.insert(test_node(PathPattern::Literal("b".into())));
        children.insert(test_node(PathPattern::regex_path("^user[0-9]+$")));
        children.insert(test_node(PathPattern::Any));
        children.insert(test_node(PathPattern::AnyPath));

        assert_eq!(children.all_nodes().len(), 5);
    }

    #[test]
    fn children_display_string_lists_patterns() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        children.insert(test_node(PathPattern::Literal("home".into())));
        children.insert(test_node(PathPattern::regex_path("^user[0-9]+$")));
        let display = children.display_string();
        assert!(display.contains("Literal: home"));
        assert!(display.contains("Regex: ^user[0-9]+$"));
    }

    #[test]
    fn children_match_literal_returns_exact_child() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let literal = test_node(PathPattern::Literal("home".into()));
        children.insert(literal.clone());

        let found = children.match_literal("home").unwrap();
        assert!(Arc::ptr_eq(&found, &literal));
    }

    #[test]
    fn children_match_regex_returns_first_matching_child() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let regex = test_node(PathPattern::regex_path("^user[0-9]+$"));
        children.insert(regex.clone());

        let found = children.match_regex("user42").unwrap();
        assert!(Arc::ptr_eq(&found, &regex));
    }

    #[test]
    fn children_match_any_returns_wildcard_child() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let any = test_node(PathPattern::Any);
        children.insert(any.clone());

        let found = children.match_any().unwrap();
        assert!(Arc::ptr_eq(&found, &any));
    }

    #[test]
    fn children_match_any_path_returns_catch_all_child() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let any_path = test_node(PathPattern::AnyPath);
        children.insert(any_path.clone());

        let found = children.match_any_path().unwrap();
        assert!(Arc::ptr_eq(&found, &any_path));
    }

    #[test]
    fn children_match_one_segment_ignores_any_path() {
        let mut children = Children::<TestContext, TcpTransport>::new();
        let any_path = test_node(PathPattern::AnyPath);
        children.insert(any_path);

        assert!(children.match_one_segment("home").is_none());
    }

    #[test]
    fn children_match_regex_with_idx_skips_previous_matches() {
        let children = Children::<TestContext, TcpTransport>::new();
        let first = test_node(PathPattern::regex_path("^user.*$"));
        let second = test_node(PathPattern::regex_path("^user42$"));
        children.insert(first);
        children.insert(second.clone());

        let (found, idx) = children.match_regex_with_idx("user42", 1).unwrap();
        assert_eq!(idx, 1);
        assert!(Arc::ptr_eq(&found, &second));
    }

    #[test]
    fn children_match_step_returns_resume_state_after_literal() {
        let children = Children::<TestContext, TcpTransport>::new();
        let literal = test_node(PathPattern::Literal("home".into()));
        children.insert(literal.clone());

        let (found, state) = children.match_step("home", PartialState::NotStart);
        assert!(Arc::ptr_eq(&found.unwrap(), &literal));
        assert_eq!(state, PartialState::Lit);
    }

    #[test]
    fn children_match_step_resumes_after_regex() {
        let children = Children::<TestContext, TcpTransport>::new();
        let first = test_node(PathPattern::regex_path("^user.*$"));
        let second = test_node(PathPattern::regex_path("^user42$"));
        children.insert(first.clone());
        children.insert(second.clone());

        let (found, state) = children.match_step("user42", PartialState::NotStart);
        assert!(Arc::ptr_eq(&found.unwrap(), &first));
        assert_eq!(state, PartialState::Reg(0));

        let (found, state) = children.match_step("user42", state);
        assert!(Arc::ptr_eq(&found.unwrap(), &second));
        assert_eq!(state, PartialState::Reg(1));
    }
}
