//! Parsed access-point definitions used by the procedural macros.
//!
//! This mirrors `hotaru_core::executable::def::AccessPointDef`, but stores
//! syntax-level values rather than runtime handlers and configuration.

mod address;
mod counter;
mod endpoint;
mod final_handler;
mod handler;
mod outpoint;
mod outpoint_mw;
mod parsed;
mod parts;

pub(crate) use address::{RouteAddress, UrlMode};
pub(crate) use counter::next_anonymous_ident;
pub(crate) use endpoint::Endpoint;
pub(crate) use final_handler::FinalHandler;
pub(crate) use handler::APHandlerDef;
pub(crate) use outpoint::Outpoint;
pub(crate) use outpoint_mw::OutpointMW;
pub(crate) use parsed::ParsedAP;
pub(crate) use parts::APParts;
