pub use crate::PathPattern;
pub use crate::Url;
pub use crate::Value;
pub use crate::object;
pub use crate::{AnyPath, AnyUrl, LitUrl, RegUrl, TrailingSlash};
pub use crate::{Client, RunMode, Server, TcpTransport};
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
pub use crate::ahttpm::akari_json;
pub use crate::ahttpm::akari_render;
pub use crate::{Locals, LocalsClone, Params, ParamsClone}; // Always keep this in prelude 

pub use std::sync::Arc;
pub use std::thread::sleep;
pub use std::time::Duration;
pub use tokio;

pub type SServer<TS = TcpTransport> = Lazy<Arc<Server<TS>>>;
pub type SClient<TS = TcpTransport> = Lazy<Arc<Client<TS>>>;
pub type SUrl<C> = Lazy<Arc<Url<C>>>;
pub type SPattern = Lazy<PathPattern>;
