//! Inbound behavior and synchronous entry helpers for unified application targets.

use crate::prelude::Arc;
#[cfg(not(feature = "std"))]
use crate::prelude::*;
use core::any::TypeId;
use core::panic;
use core::time::Duration;

use crate::app::instance::{
    App,
    target::{InboundOnly, InboundTarget},
};
use crate::app::runtime::{Either, OnceCellCap, RuntimeSpec};
use crate::marker::{MaybeSend, MaybeSendSync};
use crate::{debug_error, debug_log, debug_warn};

use crate::connection::{Inbound, TransportSpec};
use crate::protocol::{Protocol, RequestContext};
use crate::url::UrlError;

pub use crate::app::registry::ProtocolRegistryKind;
pub use crate::executable::ProtocolRegistryBuilder;

// use crate::app::middleware::AsyncMiddleware;
pub use crate::app::common::builder::AppBuilder;
use crate::app::common::builder::ServerRole;
use crate::app::common::{RunMode, TimeoutSetting};

// type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Server runtime for inbound protocol traffic.
pub type Server<TS, Rt> = App<TS, Rt, InboundOnly>;

impl<TS: TransportSpec, Rt: RuntimeSpec> App<TS, Rt, InboundOnly> {
    /// Creates a server builder whose terminal method is `build()`.
    pub fn new() -> AppBuilder<ServerRole, TS, Rt> {
        AppBuilder::new()
    }

    /// Synthetically invokes a registered endpoint by name without opening a
    /// wire. This diagnostic helper is intentionally server-role-specific.
    pub async fn request_fn<P>(
        self: &Arc<Self>,
        name: &str,
        request: <P::Context as RequestContext>::Request,
    ) -> Result<
        Result<<P::Context as RequestContext>::Response, <P::Context as RequestContext>::Error>,
        UrlError,
    >
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
    {
        let entry = self
            .registry
            .entry::<P>()
            .ok_or(UrlError::ProtocolNotFound)?;
        let access_point = entry
            .access_points
            .get(name)
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;
        let node = access_point
            .resolve()
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;

        let mut context = P::Context::default();
        context.inject_request(request);

        let response = match node.run(context).await {
            Ok(context) => Ok(context.into_response()),
            Err(error) => Err(error),
        };
        Ok(response)
    }
}

impl<TS, Rt, T> App<TS, Rt, T>
where
    TS: TransportSpec,
    Rt: RuntimeSpec,
    T: InboundTarget<TS, Rt>,
    T::Outbound<TS, Rt>: MaybeSendSync,
{
    // TODO : implement this method
    // pub fn get_protocol_address<P: Protocol>(&self) -> String {
    //     unimplemented!()
    // }

    pub fn set_max_connection_time(&mut self, max_connection_time: TimeoutSetting) {
        self.config.set_max_connection_time(max_connection_time);
    }

    pub fn set_max_frame_process_timeout(&mut self, timeout: TimeoutSetting) {
        self.config.set_max_frame_process_timeout(timeout);
    }

    pub fn get_max_frame_process_timeout(&self) -> TimeoutSetting {
        self.config.max_frame_process_timeout()
    }

    /// Get the default protocol type (first registered protocol)
    /// TODO: What happen when empty - Should not return ()'s Type ID!
    pub fn default_protocol_type(self: &Arc<Self>) -> TypeId {
        // Return the first protocol's TypeId from registry
        self.registry
            .first_protocol_type_id()
            .unwrap_or_else(|| TypeId::of::<()>())
    }

    // TODO: Implement register_from on Url or remove this method
    // pub fn reg_from<P: Protocol + 'static>(self: &Arc<Self>, segments: &[PathPattern]) -> Arc<Url<P::Context>> {
    //     match self.registry.reg_from::<P>(segments) {
    //         Ok(url) => url,
    //         Err(e) => {
    //             eprintln!("{}", e);
    //             dangling_url()
    //         }
    //     }
    // }

    /// Handle one accepted wire stream.
    pub fn handle_wire(self: Arc<Self>, conn: TS::Wire) {
        // Resolve Inherit to the protocol's own default before spawning.
        let timeout = match self.config.max_connection_time() {
            TimeoutSetting::Inherit => self.registry.default_connection_timeout(),
            TimeoutSetting::Disabled => None,
            TimeoutSetting::Fixed(d) => Some(d),
        };
        let app = self.clone();
        Rt::spawn_detached(async move {
            match timeout {
                None => {
                    self.registry.serve(app.runtime.clone(), conn).await;
                }
                Some(duration) => {
                    match Rt::select2(
                        self.registry.serve(app.runtime.clone(), conn),
                        Rt::sleep(duration),
                    )
                    .await
                    {
                        Either::Left(_) => {}
                        Either::Right(_) => {
                            debug_warn!("⚠️ Connection timed out after {:?}", duration);
                        }
                    }
                }
            }
        });
    }

    /// Run the application until this runtime's default stop condition fires.
    ///
    /// `run()` is executor-neutral: it does not create a Tokio runtime or any
    /// other executor. The caller (or a future `hotaru::main` macro) is
    /// responsible for driving this future. For Tokio, the default stop
    /// condition is Ctrl+C; for runtimes without a default stop source it may
    /// be `pending()` forever.
    ///
    /// Example:
    /// ```ignore
    /// use hotaru_core::app::server::Server;
    /// use hotaru_io_tokio::TcpTransport;
    /// use hotaru_rt_tokio::TokioRuntime;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let app = Server::<TcpTransport, TokioRuntime>::new()
    ///         .worker(4)  // Server will use 4 worker threads
    ///         .build();
    ///     app.run().await;
    /// }
    /// ```
    pub async fn run(self: Arc<Self>) {
        self.run_until(Rt::default_stop()).await;
    }

    /// Run the application until `stop` resolves.
    ///
    /// This is the runtime-neutral accept loop. It uses `Rt::select2`,
    /// `Rt::sleep`, and `Rt::spawn_detached`; no Tokio APIs are referenced.
    /// Provide any stop source you want: OS signal, board interrupt, supervisor
    /// message, deadline, or `core::future::pending()` for never-stop service.
    pub async fn run_until<S>(self: Arc<Self>, stop: S)
    where
        S: core::future::Future<Output = ()> + MaybeSend,
    {
        let inbound = self
            .ensure_inbound()
            .await
            .unwrap_or_else(|_| panic!("Failed to bind inbound transport"))
            .clone();

        debug_log!("Inbound transport bound");

        let mut stop = core::pin::pin!(stop);

        loop {
            match Rt::select2(inbound.accept(), &mut stop).await {
                Either::Left(Ok(conn)) => {
                    debug_log!("Accepted inbound wire");
                    Arc::clone(&self).handle_wire(conn);
                }
                Either::Left(Err(_e)) => {
                    if self.get_mode() == RunMode::Build {
                        debug_error!("Failed to accept connection: {_e}");
                    }
                }
                Either::Right(()) => {
                    debug_log!("Shutting down server...");
                    break;
                }
            }
        }

        Rt::sleep(Duration::from_secs(1)).await;
        debug_log!("Server shutdown complete");
    }

    /// Returns the `TS::Inbound` instance, binding on first use.
    pub async fn ensure_inbound(&self) -> Result<&Arc<TS::Inbound>, TS::IoError> {
        self.inbound_state
            .inbound
            .get_or_try_init(|| async {
                Ok(Arc::new(
                    TS::Inbound::bind(self.inbound_state.binding.clone()).await?,
                ))
            })
            .await
    }
}

// ============================================================================
// Sync-entry helpers behind the `run_server*!` macros.
//
// - `run_server` / `run_server_until` — build the runtime, block the current
//   thread (require `BlockingRuntimeCap`).
// - `run_server_no_block` / `run_server_no_block_until` — spawn detached on
//   the *already-running* runtime.
// ============================================================================

use crate::app::runtime::BlockingRuntimeCap;

/// Blocks until `Rt::default_stop()` fires (Ctrl+C under tokio).
pub fn run_server<TS, Rt>(server: Arc<Server<TS, Rt>>)
where
    TS: TransportSpec,
    Rt: BlockingRuntimeCap,
{
    Rt::block_on(async move {
        server.run_until(Rt::default_stop()).await;
    });
}

/// Blocks until user-supplied `stop` future resolves.
pub fn run_server_until<TS, Rt, S>(server: Arc<Server<TS, Rt>>, stop: S)
where
    TS: TransportSpec,
    Rt: BlockingRuntimeCap,
    S: core::future::Future<Output = ()> + MaybeSend + 'static,
{
    Rt::block_on(async move {
        server.run_until(stop).await;
    });
}

/// Fire-and-forget on the active runtime; uses `Rt::default_stop()`.
pub fn run_server_no_block<TS, Rt>(server: Arc<Server<TS, Rt>>)
where
    TS: TransportSpec,
    Rt: RuntimeSpec,
{
    Rt::spawn_detached(async move {
        server.run_until(Rt::default_stop()).await;
    });
}

/// Fire-and-forget on the active runtime with user-supplied stop.
pub fn run_server_no_block_until<TS, Rt, S>(server: Arc<Server<TS, Rt>>, stop: S)
where
    TS: TransportSpec,
    Rt: RuntimeSpec,
    S: core::future::Future<Output = ()> + MaybeSend + 'static,
{
    Rt::spawn_detached(async move {
        server.run_until(stop).await;
    });
}
