//! HTTP-specific re-exports for standard Hotaru HTTP implementation
//! 
//! Use this module when working with the standard HTTP protocol:
//! ```rust
//! use hotaru::http::*;
//! ```

// Re-export HTTP protocol and context
pub use crate::{HTTP, HTTP_CLIENT, HttpContext, HttpClientContext};
// Re-export type alias for backward compatibility
pub use hotaru_core::http::context::HttpReqCtx;
// HttpResCtx was for client-side in old Starberry - use HttpContext for both now
pub type HttpResCtx = HttpContext; 

// Re-export HTTP types
pub use hotaru_core::http::response::HttpResponse;  
pub use hotaru_core::http::request::HttpRequest;
pub use hotaru_core::http::safety::HttpSafety;
pub use hotaru_core::http::http_value::HttpMethod::*; 
pub use hotaru_core::http::meta::*; 
pub use hotaru_core::http::http_value::*; 
pub use hotaru_core::http::cookie::*; 
pub use hotaru_core::http::body::*; 
pub use hotaru_core::http::form::*; 
pub use hotaru_core::http::encoding::*;

// Re-export request and response templates
pub use crate::request_templates;
pub use crate::response_templates;

// For convenience, also re-export the template functions directly
pub use crate::request_templates::*;
pub use crate::response_templates::*;
