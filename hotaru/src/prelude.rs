pub use once_cell::sync::Lazy; 
pub use crate::Value;  
pub use crate::object;  
pub use crate::{App, RunMode}; 
pub use crate::{LitUrl, RegUrl, AnyUrl, AnyPath, TrailingSlash}; 
pub use crate::PathPattern; 
pub use crate::Url; 
pub use crate::{ProtocolHandlerBuilder as ProtocolBuilder, ProtocolRegistryBuilder as HandlerBuilder, ProtocolRegistryKind}; 

// Core protocol traits (protocol-agnostic)
pub use crate::{Protocol, RequestContext, ProtocolRole};

// Macros
pub use crate::endpoint;
pub use crate::middleware;
pub use crate::{LApp, LUrl, LPattern}; 

// Template rendering (protocol-agnostic)
pub use crate::ahttpm::akari_render; 
pub use crate::ahttpm::akari_json; 
pub use crate::AsyncMiddleware; 
pub use crate::{Params, ParamsClone, Locals, LocalsClone}; // Always keep this in prelude 

pub use std::sync::Arc; 
pub use std::thread::sleep; 
pub use std::time::Duration; 
pub use tokio; 

pub type SApp = Lazy<Arc<App>>; 
pub type SUrl<C> = Lazy<Arc<Url<C>>>; 
pub type SPattern = Lazy<PathPattern>; 
