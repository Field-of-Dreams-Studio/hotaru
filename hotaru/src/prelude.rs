//! Curated prelude — `use hotaru::prelude::*;` brings in `Server`,
//! `Client`, the `endpoint!` / `outpoint!` / `middleware!` macros, the
//! lazy-static `S*` aliases, and the core protocol traits.

pub use crate::PathPattern;
pub use crate::Url;
pub use crate::Value;
pub use crate::object;
pub use crate::{AnyPath, AnyUrl, LitUrl, RegUrl, TrailingSlash};
pub use crate::{Client, RunMode, Server, TcpTransport, TimeoutSetting};
pub use crate::{Inbound, Outbound};
pub use crate::{
    ProtocolHandlerBuilder as ProtocolBuilder, ProtocolRegistryBuilder as HandlerBuilder,
    ProtocolRegistryKind,
};
pub use once_cell::sync::Lazy;

// Core protocol traits (protocol-agnostic)
pub use crate::{EmptyError, Protocol, ProtocolError, ProtocolRole, RequestContext};

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
pub use tokio;

/// Lazy-static `Arc<Server<TS>>` — pair with `LServer!` to declare
/// a process-wide server.
pub type SServer<TS = TcpTransport> = Lazy<Arc<Server<TS>>>;
/// Lazy-static `Arc<Client<TS>>` — pair with `LClient!` to declare
/// a process-wide outbound client.
pub type SClient<TS = TcpTransport> = Lazy<Arc<Client<TS>>>;
/// Lazy-static `Arc<Url<C>>` — pair with `LUrl!` to declare a
/// process-wide registered URL node.
pub type SUrl<C> = Lazy<Arc<Url<C>>>;
/// Lazy-static `PathPattern` — pair with `LPattern!` to declare a
/// process-wide compiled URL pattern.
pub type SPattern = Lazy<PathPattern>;
