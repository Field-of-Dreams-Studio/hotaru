//! Curated prelude — `use hotaru::prelude::*;` brings in `Server`,
//! `Client`, the `endpoint!` / `outpoint!` / `middleware!` macros, the
//! lazy-static `S*` aliases, and the core protocol traits.

#[cfg(feature = "io_embedded")]
pub use crate::EmbeddedIo;
#[cfg(feature = "io_futures")]
pub use crate::FuturesIo;
pub use crate::PathPattern;
#[cfg(feature = "tokio")]
pub use crate::TokioRuntime;
pub use crate::Url;
pub use crate::Value;
pub use crate::object;
pub use crate::{AnyPath, AnyUrl, LitUrl, RegUrl, TrailingSlash};
pub use crate::{Client, RunMode, Server, TimeoutSetting};
pub use crate::{Inbound, Outbound};
pub use crate::{
    ProtocolHandlerBuilder as ProtocolBuilder, ProtocolRegistryBuilder as HandlerBuilder,
    ProtocolRegistryKind,
};
#[cfg(feature = "io_tokio")]
pub use crate::{TcpTransport, TokioIo};
pub use once_cell::sync::Lazy;

// Core protocol traits (protocol-agnostic)
pub use crate::{
    EmptyError, EndpointOutcome, Protocol, ProtocolError, ProtocolRole, RequestContext,
};

// Macros
pub use crate::call;
pub use crate::endpoint;
pub use crate::middleware;
pub use crate::outpoint;
pub use crate::run;
pub use crate::{LClient, LPattern, LServer, LUrl};

// Template rendering (protocol-agnostic)
pub use crate::AsyncMiddleware;
#[cfg(feature = "http")]
pub use crate::ahttpm::akari_json;
#[cfg(feature = "http")]
pub use crate::ahttpm::akari_render;
pub use crate::{Locals, LocalsClone, Params, ParamsClone}; // Always keep this in prelude

pub use std::sync::Arc;
pub use std::thread::sleep;
pub use std::time::Duration;
#[cfg(feature = "tokio")]
pub use tokio;

/// Lazy-static `Arc<Server<TS>>` — pair with `LServer!` to declare
/// a process-wide server.
#[cfg(feature = "tokio")]
pub type SServer<TS = TcpTransport, Rt = TokioRuntime> = Lazy<Arc<Server<TS, Rt>>>;
#[cfg(not(feature = "tokio"))]
pub type SServer<TS, Rt> = Lazy<Arc<Server<TS, Rt>>>;
/// Lazy-static `Arc<Client<TS>>` — pair with `LClient!` to declare
/// a process-wide outbound client.
#[cfg(feature = "tokio")]
pub type SClient<TS = TcpTransport, Rt = TokioRuntime> = Lazy<Arc<Client<TS, Rt>>>;
#[cfg(not(feature = "tokio"))]
pub type SClient<TS, Rt> = Lazy<Arc<Client<TS, Rt>>>;
/// Lazy-static `Arc<Url<C>>` — pair with `LUrl!` to declare a
/// process-wide registered URL node.
#[cfg(feature = "io_tokio")]
pub type SUrl<C, TS = TcpTransport> = Lazy<Arc<Url<C, TS>>>;
#[cfg(not(feature = "io_tokio"))]
pub type SUrl<C, TS> = Lazy<Arc<Url<C, TS>>>;
/// Lazy-static `PathPattern` — pair with `LPattern!` to declare a
/// process-wide compiled URL pattern.
pub type SPattern = Lazy<PathPattern>;
