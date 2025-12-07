//! Prelude for h2per - convenient imports for using Hyper with Hotaru
//! 
//! Import everything you need with:
//! ```rust
//! use h2per::prelude::*;
//! ```

// Re-export protocol types
pub use crate::{HYPER1, HYPER2, HYPER3};
pub use crate::protocol::{HyperHttp1, HyperHttp2, HyperHttp3};

// Re-export context types  
pub use crate::context::{HyperContext, HyperRequest, HyperResponse, HttpVersion, Body, switch_protocol_response};

// Re-export request and response templates
pub use crate::request::request_templates::*;
pub use crate::response::response_templates::*;
pub use crate::request::RequestExt;

// Re-export commonly used hyper types
pub use hyper::{Request, Response, Method, StatusCode, Version};
pub use http::header::{HeaderMap, HeaderName, HeaderValue};

// Re-export body utilities
pub use bytes::{Bytes, BytesMut};
pub use http_body_util::{Full, Empty, BodyExt};

// Re-export from hotaru_core for Protocol trait
pub use hotaru_core::connection::{Protocol, RequestContext, ProtocolRole};