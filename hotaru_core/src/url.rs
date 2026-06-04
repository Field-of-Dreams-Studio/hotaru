// pub mod segments;
pub mod error;
pub mod node;
pub mod parser;
pub mod pattern;
pub mod root;

// pub use self::segments::{Url, dangling_url};
pub use self::error::UrlError;
pub use self::node::{
    Children, ChildrenInner, FrameNode, LiteralChild, RegexChild, StepName, UrlNode, WalkCursor,
    WalkFrame,
};
pub use self::pattern::{PathPattern, RegexSegment, path_pattern_creator::*};
pub use self::root::UrlRegistration;
pub use self::root::UrlRoot;
