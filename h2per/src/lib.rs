//! Hyper-based HTTP/1, HTTP/2, and HTTP/3 protocol implementations for Hotaru.
//!
//! This crate provides Protocol trait implementations for all HTTP versions
//! using the hyper library as the underlying engine.

pub mod context;
pub mod hyper_exports;
mod io_compat;
pub mod message;
pub mod prelude;
pub mod protocol;
pub mod request;
pub mod response;
mod service;
pub mod stream;
pub mod transport;
pub mod upgrade;
pub mod websocket;

// Re-export protocol implementations
pub use context::{HyperContext, HyperRequest, HyperResponse};
pub use protocol::{HyperHttp1, HyperHttp2, HyperHttp3};

// Type aliases to distinguish from core HTTP implementation
pub type HYPER1 = HyperHttp1;
pub type HYPER2 = HyperHttp2;
pub type HYPER3 = HyperHttp3;

// Re-export commonly used Hyper types for convenience
pub use hyper_exports::{
    Body, HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode, Uri, Version,
};
