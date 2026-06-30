use alloc::sync::Arc;
use core::marker::PhantomData;

use akari::extensions::ParamsClone;

use crate::{
    app::common::{
        AppBuilder, OperationalConfig, RunMode, RuntimeConfig, TimeoutSetting, builder::ClientRole,
    },
    app::runtime::{Either, OnceCellCap, RuntimeSpec},
    connection::{Outbound, TransportSpec},
    executable::ExecutableBinding,
    protocol::Protocol,
    protocol::{Channel, RequestContext},
    url::{PathPattern, UrlError, UrlNode, UrlRoot, node::StepName},
};

pub use crate::app::registry::ProtocolRegistryKind;

/// Outbound runtime for protocol-routed requests.
pub struct Client<TS: TransportSpec, Rt: RuntimeSpec> {
    pub registry: ProtocolRegistryKind<TS>,
    pub target: <TS::Outbound as Outbound>::ConnectTarget,
    /// Built `TS::Outbound`, materialized on first `ensure_outbound`.
    pub outbound: <Rt as RuntimeSpec>::OnceCell<Arc<TS::Outbound>>,
    pub runtime: Arc<RuntimeConfig>,
    pub config: OperationalConfig,
    pub(crate) _rt: PhantomData<fn() -> Rt>,
}

impl<TS: TransportSpec, Rt: RuntimeSpec> Client<TS, Rt> {
    /// Creates a client builder whose terminal method is `build()`.
    pub fn new() -> AppBuilder<ClientRole, TS, Rt> {
        AppBuilder::new()
    }

    /// Returns the registered root URL tree for one protocol.
    pub fn root<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        &self,
    ) -> Option<Arc<UrlRoot<P::Context, TS>>> {
        self.registry.url::<P>()
    }

    /// Register a literal outpoint URL — no pattern grammar; the string is
    /// split on `/` and each segment becomes a literal `PathPattern`.
    /// `name` identifies the outpoint for later lookup (used by
    /// `request_fn` / `run_fn` / `call_fn`).
    pub fn lit_url<P, T, N>(
        self: &Arc<Self>,
        url: T,
        name: N,
        executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<(), UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        T: AsRef<str>,
        N: Into<String>,
    {
        let url = url.as_ref();
        let path: Vec<PathPattern> = if url.is_empty() {
            Vec::new()
        } else {
            url.split('/').map(PathPattern::literal_path).collect()
        };
        self.registry
            .register::<P, _>(name, path, StepName::default(), executable, config)?;
        Ok(())
    }

    /// Register an outpoint URL using Hotaru pattern syntax (literals,
    /// `<name>`, `<type:name>`, `<regex>`, `*`, `**path`). `name` identifies
    /// the outpoint for later lookup.
    pub fn url<P, T, N>(
        self: &Arc<Self>,
        url: T,
        name: N,
        executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<(), UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        T: AsRef<str>,
        N: Into<String>,
    {
        let tokens = P::tokenize_url(url.as_ref())?;
        let (path, step_names) = crate::url::tokens_to_patterns(&tokens)?;
        self.registry
            .register::<P, _>(name, path, step_names.into(), executable, config)?;
        Ok(())
    }

    /// Returns the shared runtime mode.
    pub fn get_mode(self: &Arc<Self>) -> RunMode {
        self.runtime.mode()
    }

    /// Returns the shared runtime config store.
    pub fn config(self: &Arc<Self>) -> &crate::extensions::Params {
        self.runtime.config()
    }

    /// Returns the shared runtime statics store.
    pub fn statics(self: &Arc<Self>) -> &crate::extensions::Locals {
        self.runtime.statics()
    }

    /// Gets one typed config value from the shared runtime store.
    pub fn get_config<T: Clone + Send + Sync + 'static>(self: &Arc<Self>) -> Option<T> {
        self.runtime.get_config::<T>()
    }

    /// Gets one typed static value from the shared runtime store.
    pub fn get_static<T: Clone + Send + Sync + 'static>(self: &Arc<Self>, key: &str) -> Option<T> {
        self.runtime.get_static::<T>(key)
    }

    /// Gets one typed config value or its default.
    pub fn get_config_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
    ) -> T {
        self.runtime.get_config::<T>().unwrap_or_default()
    }

    /// Gets one typed static value or its default.
    pub fn get_static_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
        key: &str,
    ) -> T {
        self.runtime.get_static::<T>(key).unwrap_or_default()
    }

    /// Returns the configured client worker count.
    ///
    /// Client worker scheduling is not implemented yet. This currently only
    /// exposes the configured value; it does not create a client worker pool.
    pub fn get_worker(self: &Arc<Self>) -> usize {
        self.config.worker()
    }

    /// Returns the configured maximum connection lifetime setting.
    pub fn get_max_connection_time(self: &Arc<Self>) -> TimeoutSetting {
        self.config.max_connection_time()
    }

    /// Returns the configured maximum request processing time in seconds.
    pub fn get_max_frame_process_time(self: &Arc<Self>) -> usize {
        self.config.max_frame_process_time()
    }

    /// Returns the `TS::Outbound` instance, building on first use.
    pub async fn ensure_outbound(self: &Arc<Self>) -> Result<&Arc<TS::Outbound>, TS::IoError> {
        self.outbound
            .get_or_try_init(|| async {
                Ok(Arc::new(TS::Outbound::build(self.target.clone()).await?))
            })
            .await
    }

    /// Opens one outbound wire to this client's configured target.
    pub async fn connect(self: &Arc<Self>) -> Result<TS::Wire, TS::IoError> {
        self.ensure_outbound().await?.connect().await
    }

    /// Runs protocol-side client handling on an existing wire.
    pub async fn run_wire(self: &Arc<Self>, wire: TS::Wire) {
        self.registry.request(self.runtime.clone(), wire).await;
    }

    /// Resolves an outbound path into a concrete endpoint node.
    pub async fn resolve<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
    ) -> Result<Arc<UrlNode<P::Context, TS>>, UrlError> {
        let Some(root) = self.root::<P>() else {
            return Err(UrlError::InvalidPath(path.to_string()));
        };

        root.walk_str(path)
            .await
            .ok_or_else(|| UrlError::InvalidPath(path.to_string()))
    }

    /// Resolves an outbound path with an explicit depth limit.
    pub async fn resolve_with_limit<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
        max_depth: u32,
    ) -> Result<Arc<UrlNode<P::Context, TS>>, UrlError> {
        let Some(root) = self.root::<P>() else {
            return Err(UrlError::InvalidPath(path.to_string()));
        };

        root.walk_str_with_limit(path, max_depth)
            .await
            .ok_or_else(|| UrlError::InvalidPath(path.to_string()))
    }

    /// Executes one outbound route by path and runs its middleware/final handler chain.
    pub async fn request<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
        ctx: P::Context,
    ) -> Result<Result<P::Context, <P::Context as RequestContext>::Error>, UrlError> {
        let outpoint = self.resolve::<P>(path).await?;
        Ok(outpoint.run(ctx).await)
    }

    /// Executes one outbound route by path with an explicit depth limit.
    pub async fn request_with_limit<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
        max_depth: u32,
        ctx: P::Context,
    ) -> Result<Result<P::Context, <P::Context as RequestContext>::Error>, UrlError> {
        let outpoint = self.resolve_with_limit::<P>(path, max_depth).await?;
        Ok(outpoint.run(ctx).await)
    }

    /// Run a named outpoint: look up the access point, open a wire, build the
    /// context, run its chain, return the response. Outer `UrlError` covers
    /// "name not found"; inner protocol error covers I/O and chain failures.
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
        let ap = entry
            .access_points
            .get(name)
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;
        let node = ap
            .resolve()
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;

        // Inner: connect-IO + chain errors land in CtxError<P>.
        let inner: Result<_, <P::Context as RequestContext>::Error> = async {
            let outbound = self.ensure_outbound().await?.clone();
            let channel = entry
                .protocol
                .acquire_channel(&self.runtime, outbound)
                .await?;

            let mut ctx = P::Context::default();
            P::install_channel(&mut ctx, channel);
            ctx.inject_request(request);

            let ctx = node.run(ctx).await?;
            Ok(ctx.into_response())
        }
        .await;

        Ok(inner)
    }

    /// Spawn a persistent call task: one outpoint, one channel, looped while
    /// the channel stays open. Lookup errors surface as `UrlError`; runtime
    /// errors land in the join handle's inner result.
    pub async fn call_fn<P>(
        self: &Arc<Self>,
        name: &str,
    ) -> Result<Rt::JoinHandle<Result<(), <P::Context as RequestContext>::Error>>, UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        <P::Context as RequestContext>::Error: Send + 'static,
    {
        let entry = self
            .registry
            .entry::<P>()
            .ok_or(UrlError::ProtocolNotFound)?;
        let ap = entry
            .access_points
            .get(name)
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;
        let node = ap
            .resolve()
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;

        Ok(self.spawn_call_loop::<P>(entry.protocol.clone(), node))
    }

    /// Same as `call_fn`, addressed by path instead of name. Resolves once
    /// before the loop; subsequent root-endpoint rebinds are not picked up.
    pub async fn call_url<P>(
        self: &Arc<Self>,
        path: &str,
    ) -> Result<Rt::JoinHandle<Result<(), <P::Context as RequestContext>::Error>>, UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        <P::Context as RequestContext>::Error: Send + 'static,
    {
        let node = self.resolve::<P>(path).await?;
        let entry = self
            .registry
            .entry::<P>()
            .ok_or(UrlError::ProtocolNotFound)?;

        Ok(self.spawn_call_loop::<P>(entry.protocol.clone(), node))
    }

    /// Drive one outpoint over one channel until the channel closes, an
    /// error fires, or `max_connection_time` expires.
    fn spawn_call_loop<P>(
        self: &Arc<Self>,
        protocol: P,
        node: Arc<UrlNode<P::Context, TS>>,
    ) -> Rt::JoinHandle<Result<(), <P::Context as RequestContext>::Error>>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        <P::Context as RequestContext>::Error: Send + 'static,
    {
        let this = self.clone();
        let deadline_setting = self.config.max_connection_time();

        Rt::spawn(async move {
            // Acquire the channel inside the task so I/O errors fall into
            // the join handle's inner result, not the outer UrlError.
            let outbound = this.ensure_outbound().await?.clone();
            let channel = protocol.acquire_channel(&this.runtime, outbound).await?;

            // One ctx, reused across iterations; channel stays installed.
            let mut ctx = <P::Context as Default>::default();
            P::install_channel(&mut ctx, channel.clone());

            let deadline = match deadline_setting {
                TimeoutSetting::Fixed(d) => Some(Rt::instant_plus(Rt::now(), d)),
                _ => None,
            };

            while channel.is_open() {
                let iter = node.run(ctx);
                ctx = match deadline {
                    Some(d) => match Rt::select2(iter, Rt::sleep_until(d)).await {
                        Either::Left(result) => result?,
                        Either::Right(_) => {
                            channel.close();
                            break;
                        }
                    },
                    None => iter.await?,
                };
            }
            Ok(())
        })
    }
}
