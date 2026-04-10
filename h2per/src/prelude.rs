//! Prelude for h2per - convenient imports for using Hyper with Hotaru
//!
//! Import everything you need with:
//! ```rust
//! use h2per::prelude::*;
//! ```

// Re-export protocol types
pub use crate::protocol::{HyperHttp1, HyperHttp2, HyperHttp3};
pub use crate::{HYPER1, HYPER2, HYPER3};

// Re-export context types
pub use crate::context::{
    Body, HttpVersion, HyperContext, HyperRequest, HyperResponse, switch_protocol_response,
};

// Re-export request and response templates
pub use crate::request::RequestExt;
pub use crate::request::request_templates::*;
pub use crate::response::response_templates::*;

// Re-export commonly used hyper types
pub use http::header::{HeaderMap, HeaderName, HeaderValue};
pub use hyper::{Method, Request, Response, StatusCode, Version};

// Re-export body utilities
pub use bytes::{Bytes, BytesMut};
pub use http_body_util::{BodyExt, Empty, Full};

// Re-export from hotaru_core for Protocol trait
pub use hotaru_core::connection::{Protocol, ProtocolRole, RequestContext};
