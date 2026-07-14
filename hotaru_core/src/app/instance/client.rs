//! Outbound behavior for unified application targets.

use crate::prelude::Arc;
#[cfg(not(feature = "std"))]
use crate::prelude::*;

use crate::{
    app::{
        common::{AppBuilder, TimeoutSetting, builder::ClientRole},
        instance::{
            App,
            target::{OutboundOnly, OutboundTarget},
        },
        runtime::{Either, OnceCellCap, RuntimeSpec},
    },
    connection::{Outbound, TransportSpec},
    marker::MaybeSendSync,
    protocol::Protocol,
    protocol::{Channel, RequestContext},
    url::{UrlError, UrlNode, UrlRoot},
};

pub use crate::app::registry::ProtocolRegistryKind;

/// Outbound runtime for protocol-routed requests.
pub type Client<TS, Rt> = App<TS, Rt, OutboundOnly>;

impl<TS: TransportSpec, Rt: RuntimeSpec> App<TS, Rt, OutboundOnly> {
    /// Creates a client builder whose terminal method is `build()`.
    pub fn new() -> AppBuilder<ClientRole, TS, Rt> {
        AppBuilder::new()
    }

    /// Runs a named outpoint over an acquired channel. This diagnostic helper
    /// is intentionally client-role-specific.
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

        let response: Result<_, <P::Context as RequestContext>::Error> = async {
            let outbound = self.ensure_outbound().await?.clone();
            let channel = entry
                .protocol
                .acquire_channel(&self.runtime, outbound)
                .await?;

            let mut context = P::Context::default();
            P::install_channel(&mut context, channel);
            context.inject_request(request);

            let context = node.run(context).await?;
            Ok(context.into_response())
        }
        .await;

        Ok(response)
    }
}

impl<TS, Rt, T> App<TS, Rt, T>
where
    TS: TransportSpec,
    Rt: RuntimeSpec,
    T: OutboundTarget<TS, Rt>,
    T::Inbound<TS, Rt>: MaybeSendSync,
{
    /// Returns the registered root URL tree for one protocol.
    pub fn root<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        &self,
    ) -> Option<Arc<UrlRoot<P::Context, TS>>> {
        self.registry.url::<P>()
    }

    /// Returns the configured client worker count.
    ///
    /// Client worker scheduling is not implemented yet. This currently only
    /// exposes the configured value; it does not create a client worker pool.
    pub fn get_worker(self: &Arc<Self>) -> usize {
        self.config.worker()
    }

    /// Returns the `TS::Outbound` instance, building on first use.
    pub async fn ensure_outbound(self: &Arc<Self>) -> Result<&Arc<TS::Outbound>, TS::IoError> {
        self.outbound_state
            .outbound
            .get_or_try_init(|| async {
                Ok(Arc::new(
                    TS::Outbound::build(self.outbound_state.target.clone()).await?,
                ))
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

        let segments = P::lit_parser(path);
        root.walk(segments.iter())
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

        let segments = P::lit_parser(path);
        root.walk_with_limit(segments.iter(), max_depth)
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
