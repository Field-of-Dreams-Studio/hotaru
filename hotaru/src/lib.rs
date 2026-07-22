//! Hotaru — the umbrella crate. Re-exports the public surface of
//! `hotaru_core`, `hotaru_http`, `hotaru_trans` and friends behind a
//! single import root. Most users want [`prelude`]; HTTP users also
//! want [`http`].
//!
//! # `no_std`
//!
//! Disabling `std` makes this crate `no_std` + `alloc`. The prelude's
//! `Lazy` and `S*` aliases remain std-only.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// HTTP-specific re-exports — request/response types, the `HTTP`
/// protocol alias, and (with the `https` feature) the TLS variants.
#[cfg(feature = "http")]
pub mod http;

/// Curated re-exports for typical user code: `Server`, `Client`,
/// `endpoint!` / `outpoint!` macros, the `Lazy`-wrapped `S*` aliases,
/// and the core protocol traits.
pub mod prelude;

#[cfg(feature = "tokio")]
pub use hotaru_rt_tokio::TokioRuntime;
#[cfg(feature = "embassy")]
pub use hotaru_rt_embassy;

/// Tokio-backed server alias used by the umbrella crate.
#[cfg(feature = "tokio")]
pub type Server<TS = hotaru_io_tokio::TcpTransport, Rt = hotaru_rt_tokio::TokioRuntime> =
    hotaru_core::app::server::Server<TS, Rt>;
#[cfg(not(feature = "tokio"))]
pub type Server<TS, Rt> = hotaru_core::app::server::Server<TS, Rt>;

/// Tokio-backed client alias used by the umbrella crate.
#[cfg(feature = "tokio")]
pub type Client<TS = hotaru_io_tokio::TcpTransport, Rt = hotaru_rt_tokio::TokioRuntime> =
    hotaru_core::app::client::Client<TS, Rt>;
#[cfg(not(feature = "tokio"))]
pub type Client<TS, Rt> = hotaru_core::app::client::Client<TS, Rt>;

/// Tokio-backed gateway alias for apps with inbound and outbound capabilities.
#[cfg(feature = "tokio")]
pub type Gateway<TS = hotaru_io_tokio::TcpTransport, Rt = hotaru_rt_tokio::TokioRuntime> =
    hotaru_core::app::Gateway<TS, Rt>;
#[cfg(not(feature = "tokio"))]
pub type Gateway<TS, Rt> = hotaru_core::app::Gateway<TS, Rt>;

pub use hotaru_core::app::common::{AppInUse, RunMode, TimeoutSetting};
pub use hotaru_core::app::{
    Blueprint, BlueprintError, Both, ConfiguredBlueprint, InboundOnly, OutboundOnly,
};
pub use hotaru_core::url::PathPattern;
/// Umbrella alias for Hotaru's URL routing tree.
#[cfg(feature = "io_tokio")]
pub type Url<C, TS = hotaru_io_tokio::TcpTransport> = hotaru_core::url::UrlRoot<C, TS>;
/// Umbrella alias for Hotaru's URL routing tree.
#[cfg(not(feature = "io_tokio"))]
pub type Url<C, TS> = hotaru_core::url::UrlRoot<C, TS>;
pub use hotaru_core::url::pattern::path_pattern_creator::{
    any as AnyUrl, any_path as AnyPath, literal_path as LitUrl, regex_path as RegUrl,
    trailing_slash as TrailingSlash,
};
pub use hotaru_core::url::{FrameNode, WalkCursor, WalkFrame};

/// Umbrella alias for building protocol handler registries.
#[cfg(feature = "io_tokio")]
pub type ProtocolHandlerBuilder<P, TS = hotaru_io_tokio::TcpTransport> =
    hotaru_core::executable::ProtocolEntryBuilder<P, TS>;
/// Umbrella alias for building protocol handler registries.
#[cfg(not(feature = "io_tokio"))]
pub type ProtocolHandlerBuilder<P, TS> = hotaru_core::executable::ProtocolEntryBuilder<P, TS>;
pub use hotaru_core::app::server::ProtocolRegistryKind;
pub use hotaru_core::executable::def::{
    AccessPointDef, BindError, Endpoint, EndpointHandler, FinalHandlerDef, MWChain, MWSlot,
    Outpoint, OutpointHandler, RouteAddress, UrlMode,
};
pub use hotaru_core::executable::ProtocolRegistryBuilder;
pub use hotaru_core::executable::middleware::AsyncMiddleware;
pub use hotaru_core::{debug_error, debug_trace, debug_warn};

#[cfg(feature = "template")]
pub use hotaru_core::TemplateManager;
pub use hotaru_core::Value;
pub use hotaru_core::object;

pub use hotaru_core::connection::ConnStream;
pub use hotaru_core::connection::error::{ConnectionError, Result};
pub use hotaru_core::connection::{Inbound, Outbound};
pub use hotaru_core::protocol::{
    BoxProtocolError, DefaultProtocolError, EmptyError, EndpointOutcome, Message, Protocol,
    ProtocolError, ProtocolRole, RequestContext, Stream,
};
#[cfg(feature = "io_embedded")]
pub use hotaru_io_embedded::{EmbeddedBackend, EmbeddedIo};
#[cfg(feature = "io_futures")]
pub use hotaru_io_futures::{FuturesBackend, FuturesIo};
#[cfg(feature = "io_tokio")]
pub use hotaru_io_tokio::{
    TcpAccepter, TcpConnector, TcpConnectorAddr, TcpInbound, TcpMeta, TcpOutbound, TcpStream,
    TcpTransport, TokioBackend, TokioIo,
};

pub use hotaru_core::extensions::*;

pub use akari;
pub use hotaru_core;
#[cfg(feature = "http")]
pub use hotaru_http;

pub use hotaru_trans as hrt;
pub use hrt::call;
pub use hrt::endpoint;
pub use hrt::middleware;
pub use hrt::outpoint;
pub use hrt::{params, params_clone};
pub use hrt::run;
pub use hrt::{run_server, run_server_no_block, run_server_no_block_until, run_server_until};
pub use hrt::{LClient, LPattern, LServer, LUrl};

#[cfg(not(feature = "external-ctor"))]
pub use hrt::ctor as hrt_ctor;

#[cfg(feature = "http")]
pub use ahttpm;

#[cfg(feature = "lib")]
pub use hotaru_lib;
