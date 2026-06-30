//! Hotaru — the umbrella crate. Re-exports the public surface of
//! `hotaru_core`, `hotaru_http`, `hotaru_trans` and friends behind a
//! single import root. Most users want [`prelude`]; HTTP users also
//! want [`http`].

/// HTTP-specific re-exports — request/response types, the `HTTP`
/// protocol alias, and (with the `https` feature) the TLS variants.
#[cfg(feature = "http")]
pub mod http;

/// Curated re-exports for typical user code: `Server`, `Client`,
/// `endpoint!` / `outpoint!` macros, the `Lazy`-wrapped `S*` aliases,
/// and the core protocol traits.
pub mod prelude;

pub use hotaru_core::app::runtime::TokioRuntime;

/// Tokio-backed server alias used by the umbrella crate.
pub type Server<
    TS = hotaru_core::connection::TcpTransport,
    Rt = hotaru_core::app::runtime::TokioRuntime,
> = hotaru_core::app::server::Server<TS, Rt>;

/// Tokio-backed client alias used by the umbrella crate.
pub type Client<
    TS = hotaru_core::connection::TcpTransport,
    Rt = hotaru_core::app::runtime::TokioRuntime,
> = hotaru_core::app::client::Client<TS, Rt>;
pub use hotaru_core::app::common::{RunMode, TimeoutSetting};
pub use hotaru_core::url::PathPattern;
pub type Url<C, TS = hotaru_core::connection::TcpTransport> = hotaru_core::url::UrlRoot<C, TS>;
pub use hotaru_core::url::pattern::path_pattern_creator::{
    any as AnyUrl, any_path as AnyPath, literal_path as LitUrl, regex_path as RegUrl,
    trailing_slash as TrailingSlash,
};
pub use hotaru_core::url::{FrameNode, WalkCursor, WalkFrame};

pub type ProtocolHandlerBuilder<P, TS = hotaru_core::connection::TcpTransport> =
    hotaru_core::executable::ProtocolEntryBuilder<P, TS>;
pub use hotaru_core::app::server::ProtocolRegistryKind;
pub use hotaru_core::executable::ProtocolRegistryBuilder;
pub use hotaru_core::executable::middleware::AsyncMiddleware;

pub use hotaru_core::TemplateManager;
pub use hotaru_core::Value;
pub use hotaru_core::object;

pub use hotaru_core::connection::ConnStream;
pub use hotaru_core::connection::error::{ConnectionError, Result};
pub use hotaru_core::connection::{Inbound, Outbound};
pub use hotaru_core::connection::{
    TcpAccepter, TcpConnector, TcpConnectorAddr, TcpInbound, TcpMeta, TcpOutbound, TcpTransport,
};
pub use hotaru_core::protocol::{
    BoxProtocolError, DefaultProtocolError, EmptyError, EndpointOutcome, Message, Protocol,
    ProtocolError, ProtocolRole, RequestContext, Stream,
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
pub use hrt::run;
pub use hrt::{LClient, LPattern, LServer, LUrl};

#[cfg(not(feature = "external-ctor"))]
pub use hrt::ctor as hrt_ctor;

#[cfg(feature = "http")]
pub use ahttpm;

pub use hotaru_lib;
