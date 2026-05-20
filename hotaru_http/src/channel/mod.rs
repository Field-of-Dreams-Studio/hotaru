//! HTTP channel abstractions.
//!
//! - [`HttpChannel`] — version-agnostic I/O surface (parse/send request and
//!   response) that every HTTP version must implement.
//! - [`Http1Channel`] — HTTP/1.1 implementation backed by split reader/writer
//!   halves of a `ConnStream`.

pub mod http_channel;
pub mod http1;

pub use http_channel::HttpChannel;
pub use http1::Http1Channel;
