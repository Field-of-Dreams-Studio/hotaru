use core::{any::Any, time::Duration};
use alloc::sync::Arc;

use akari::extensions::{Locals, Params};
use crate::{
    alias::PRwLock,
    app::common::RuntimeConfig,
    connection::{
        BufferedReadHalf, BufferedWriteHalf, ConnStream, MaybeSendBoxFuture, TransportSpec,
    },
};

/// Neutral protocol-entry boundary shared by server and client execution.
///
/// This trait lives in `executable` rather than under `server` or `client`
/// because the stored entry data is the same on both sides:
/// protocol instance, root URL tree, and protocol-scoped middleware defaults.
/// The role-specific difference is expressed by method families like `serve`
/// and `request`, not by duplicating the entry type itself.
pub trait ProtocolEntryTrait<TS: TransportSpec>: Send + Sync {
    /// Test if this protocol can handle the connection.
    fn test(&self, buf: &[u8]) -> bool;

    /// Handle the connection.
    fn serve(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> MaybeSendBoxFuture<'static, ()>;

    /// Handle an upgrade from another protocol.
    fn serve_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
        params: PRwLock<Params>,
        locals: PRwLock<Locals>,
    ) -> MaybeSendBoxFuture<'static, ()>;

    fn request(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> MaybeSendBoxFuture<'static, ()>;

    fn request_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
        params: PRwLock<Params>,
        locals: PRwLock<Locals>,
    ) -> MaybeSendBoxFuture<'static, ()>;

    /// Returns the protocol's default connection-timeout policy.
    ///
    /// Used to resolve [`TimeoutSetting::Inherit`] at connection time.
    fn default_connection_timeout(&self) -> Option<Duration>;

    /// Allows downcasting.
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
