// pub mod segments;
/// URL registration and routing errors.
pub mod error;
/// URL tree node types and traversal state.
pub mod node;
/// Pattern lexer/parser for route declarations.
pub mod parser;
/// Path pattern types and constructors.
pub mod pattern;
/// Root URL registry and registration APIs.
pub mod root;

// pub use self::segments::{Url, dangling_url};
pub use self::error::UrlError;
pub use self::node::{
    Children, ChildrenInner, FrameNode, LiteralChild, RegexChild, StepName, UrlNode, WalkCursor,
    WalkFrame,
};
pub use self::parser::{PatternError, RawToken, TypeKind, tokenize, tokens_to_patterns};
pub use self::pattern::{PathPattern, RegexSegment, path_pattern_creator::*};
pub use self::root::UrlRegistration;
pub use self::root::UrlRoot;
