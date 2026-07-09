//! Resumable URL-tree traversal cursor.
//!
//! Parallel surface to `walk` / `walk_str`: drains every match in
//! priority order (literal -> regex -> `*` -> `**`) via repeated
//! [`WalkCursor::find_next`] calls. Built on [`Children::match_step`];
//! existing walk paths are untouched.

#[cfg(not(feature = "std"))]
use crate::prelude::*;
use crate::prelude::Arc;

use crate::{
    connection::TransportSpec,
    protocol::RequestContext,
    url::{
        node::{Children, PartialState, UrlNode},
        root::RootNode,
    },
};

/// One in-progress child-iteration. Depth is implicit in the stack
/// index: `cursor.frames()[i]` matches `segments[i]`.
#[av::ver(unstable, since = "0.8.1", note = "Resumable URL traversal — surface may change", date = "2026-05-25")]
pub struct WalkFrame<C: RequestContext, TS: TransportSpec> {
    pub node: FrameNode<C, TS>,
    pub state: PartialState,
}

/// Which `Children` table a frame iterates. Root-rooted walks start
/// with `Root`; every deeper frame (and node-rooted walks) use `Node`.
#[av::ver(unstable, since = "0.8.1", note = "Resumable URL traversal — surface may change", date = "2026-05-25")]
pub enum FrameNode<C: RequestContext, TS: TransportSpec> {
    Root(Arc<RootNode<C, TS>>),
    Node(Arc<UrlNode<C, TS>>),
}

impl<C: RequestContext, TS: TransportSpec> FrameNode<C, TS> {
    pub fn children(&self) -> &Children<C, TS> {
        match self {
            FrameNode::Root(r) => r.children_ref(),
            FrameNode::Node(n) => n.children(),
        }
    }
}

/// Resumable depth-first traversal. Thin newtype around `Vec<WalkFrame>`.
#[av::ver(unstable, since = "0.8.1", note = "Resumable URL traversal — surface may change", date = "2026-05-25")]
pub struct WalkCursor<C: RequestContext, TS: TransportSpec> {
    frames: Vec<WalkFrame<C, TS>>,
}

impl<C, TS> WalkCursor<C, TS>
where
    C: RequestContext + Send + 'static,
    TS: TransportSpec,
{
    /// Yields `None` on every call. Used by `walk_cursor("")`.
    pub fn empty() -> Self {
        Self { frames: Vec::new() }
    }

    /// Walk from a root's children.
    pub fn from_root(root: Arc<RootNode<C, TS>>) -> Self {
        Self {
            frames: vec![WalkFrame {
                node: FrameNode::Root(root),
                state: PartialState::NotStart,
            }],
        }
    }

    /// Walk from a sub-tree node.
    pub fn from_node(node: Arc<UrlNode<C, TS>>) -> Self {
        Self {
            frames: vec![WalkFrame {
                node: FrameNode::Node(node),
                state: PartialState::NotStart,
            }],
        }
    }

    /// Next matching node, or `None` once exhausted.
    ///
    /// `segments` must be the same slice across every call for one
    /// logical walk. `AnyPath` (`<**name>`) matches are yielded
    /// terminally regardless of remaining segments — mirrors the
    /// existing `is_any_path()` guard in `walk` (issue `0.8-core-1`).
    pub fn find_next(&mut self, segments: &[&str]) -> Option<Arc<UrlNode<C, TS>>> {
        while let Some(idx) = self.frames.len().checked_sub(1) {
            let depth = idx;
            if depth >= segments.len() {
                self.frames.pop();
                continue;
            }
            let state = self.frames[idx].state;
            let segment = segments[depth];
            let (matched, next_state) =
                self.frames[idx].node.children().match_step(segment, state);
            self.frames[idx].state = next_state;

            let Some(child) = matched else {
                if next_state.is_end() {
                    self.frames.pop();
                }
                continue;
            };

            let new_depth = depth + 1;
            if new_depth == segments.len() || child.path().is_any_path() {
                return Some(child);
            }

            self.frames.push(WalkFrame {
                node: FrameNode::Node(child),
                state: PartialState::NotStart,
            });
        }
        None
    }

    /// Read-only view of the stack (for debug / breadcrumb building).
    pub fn frames(&self) -> &[WalkFrame<C, TS>] {
        &self.frames
    }
}
