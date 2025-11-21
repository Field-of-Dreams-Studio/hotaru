//! Hyper-based HTTP/1, HTTP/2, and HTTP/3 protocol implementations for Hotaru.
//!
//! This crate provides Protocol trait implementations for all HTTP versions
//! using the hyper library as the underlying engine.

pub mod protocol;
pub mod context;
pub mod transport;
pub mod stream;
pub mod message;
pub mod request;
pub mod response;
pub mod prelude;
pub mod hyper_exports;
pub mod upgrade;
pub mod websocket;
mod io_compat;
mod service;

// Re-export protocol implementations
pub use protocol::{HyperHttp1, HyperHttp2, HyperHttp3};
pub use context::{HyperContext, HyperRequest, HyperResponse};

// Type aliases to distinguish from core HTTP implementation
pub type HYPER1 = HyperHttp1;
pub type HYPER2 = HyperHttp2;
pub type HYPER3 = HyperHttp3;

// Re-export commonly used Hyper types for convenience
pub use hyper_exports::{
    Method, StatusCode, Version, Uri,
    HeaderMap, HeaderName, HeaderValue,
    Request, Response,
    Body,
};