#[cfg(feature = "http")]
pub mod http;
pub mod prelude;

pub use hotaru_core::app::client::Client;
pub use hotaru_core::app::server::Server; 
pub use hotaru_core::app::common::{RunMode, TimeoutSetting};
pub use hotaru_core::url::PathPattern;
pub use hotaru_core::url::UrlRoot as Url;
pub use hotaru_core::url::{FrameNode, WalkCursor, WalkFrame};
pub use hotaru_core::url::pattern::path_pattern_creator::{
    any as AnyUrl, any_path as AnyPath, literal_path as LitUrl, regex_path as RegUrl,
    trailing_slash as TrailingSlash,
};

pub use hotaru_core::executable::ProtocolEntryBuilder as ProtocolHandlerBuilder;
pub use hotaru_core::executable::ProtocolRegistryBuilder;
pub use hotaru_core::app::server::ProtocolRegistryKind;
pub use hotaru_core::executable::middleware::AsyncMiddleware;

pub use hotaru_core::TemplateManager;
pub use hotaru_core::Value;
pub use hotaru_core::object;

pub use hotaru_core::connection::ConnStream;
pub use hotaru_core::connection::error::{ConnectionError, Result};
pub use hotaru_core::connection::{Inbound, Outbound};
pub use hotaru_core::protocol::{
    BoxProtocolError, DefaultProtocolError, EmptyError, Message, Protocol, ProtocolError,
    ProtocolRole, RequestContext, Stream,
};
pub use hotaru_core::connection::{
    TcpAccepter, TcpConnector, TcpConnectorAddr, TcpInbound, TcpMeta, TcpOutbound, TcpTransport,
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
