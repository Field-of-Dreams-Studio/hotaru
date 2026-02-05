//! SQL protocol implementation

use std::error::Error;
use std::sync::Arc;
use async_trait::async_trait;

use crate::app::application::App;
use crate::connection::{Protocol, ProtocolRole, TcpReader, TcpWriter};
use crate::sql::{SqlContext, SqlMessage, SqlTransport};

/// SQL protocol marker type
///
/// SQL is not a traditional network protocol like HTTP.
/// It operates through connection pools rather than direct TCP streams.
#[derive(Clone)]
pub struct SQL;

#[async_trait]
impl Protocol for SQL {
    type Transport = SqlTransport;
    type Stream = ();
    type Message = SqlMessage;
    type Context = SqlContext;

    fn role(&self) -> ProtocolRole {
        ProtocolRole::Client
    }

    /// SQL doesn't use protocol detection
    fn detect(_: &[u8]) -> bool {
        false
    }

    /// SQL doesn't handle connection streams directly
    async fn handle(
        &mut self,
        _reader: TcpReader,
        _writer: TcpWriter,
        _app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
