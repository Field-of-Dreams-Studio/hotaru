use std::sync::Arc;

use crate::{
    connection::TransportSpec,
    app::common::{AppBuilder, OperationalConfig, RuntimeConfig},
}; 

pub use registry::ProtocolRegistryKind; 

pub mod registry; 

/// Client runtime placeholder during the server/client split.
pub struct Client<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    pub session: ProtocolRegistryKind<TS>,
    pub connector: TS::Connector,
    pub runtime: Arc<RuntimeConfig>,
    pub client: OperationalConfig,
}

impl<TS: TransportSpec> Client<TS> {
    pub fn new() -> AppBuilder<TS> {
        AppBuilder::new()
    }
}
