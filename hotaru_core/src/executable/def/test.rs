//! Access-point definition and binding contract tests.

use core::{
    any::Any,
    convert::Infallible,
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use crate::{
    app::{
        App, Client, InboundOnly, InboundState, OutboundOnly, OutboundState, Server,
        common::{OperationalConfig, RuntimeConfig},
        registry::ProtocolRegistryKind,
        runtime::{AsyncMutexCap, Either, OnceCellCap, RuntimeSpec},
    },
    connection::{
        MaybeSend,
        test_support::{TestMeta, TestOutbound, TestTransport, TestWire},
    },
    executable::middleware::{
        AsyncFinalHandler, AsyncMiddleware, AsyncMiddlewareChain, BoxFuture, NextFn,
    },
    marker::{MaybeSendBoxFuture, MaybeSendSync},
    prelude::{Arc, Box, Vec, vec},
    protocol::{
        Channel, DefaultProtocolError, EndpointOutcome, Protocol, ProtocolFlow, ProtocolRole,
        RequestContext,
    },
    url::{PathPattern, UrlRoot},
};

use super::{
    AccessPointDef, Endpoint, EndpointHandler, FinalHandlerDef, MWSlot, Outpoint, UrlMode,
};

#[derive(Debug)]
struct TestError;

impl core::fmt::Display for TestError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("test error")
    }
}

impl core::error::Error for TestError {}
impl DefaultProtocolError for TestError {}

impl From<Infallible> for TestError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Clone)]
enum NoChannel {}

impl Channel for NoChannel {
    fn is_open(&self) -> bool {
        match *self {}
    }

    fn close(&self) {}
}

#[derive(Default)]
struct TestResponse(Option<&'static str>);

#[derive(Default)]
struct TestCtx {
    response: TestResponse,
}

impl RequestContext for TestCtx {
    type Request = ();
    type Response = TestResponse;
    type Error = TestError;
    type Channel = NoChannel;

    fn handle_error(&mut self) {}

    fn role(&self) -> ProtocolRole {
        ProtocolRole::Server
    }

    fn inject_request(&mut self, _: ()) {}

    fn into_response(self) -> Self::Response {
        self.response
    }
}

#[derive(Clone)]
struct TestProto;

impl Protocol for TestProto {
    type Wire = TestWire;
    type TS = TestTransport;
    type Channel = NoChannel;
    type Stream = ();
    type Message = ();
    type Context = TestCtx;

    fn name(&self) -> &'static str {
        "access-point-test"
    }

    fn role(&self) -> ProtocolRole {
        ProtocolRole::Server
    }

    fn lit_parser(input: &str) -> Vec<&str> {
        input.split('/').collect()
    }

    fn detect(_: &[u8]) -> bool {
        true
    }

    fn open_channel(self, reader: TestWire, _writer: TestWire, _meta: TestMeta) -> NoChannel {
        match reader {}
    }

    fn handle(
        _channel: &NoChannel,
        _runtime: Arc<RuntimeConfig>,
        _root: Arc<UrlRoot<TestCtx, TestTransport>>,
    ) -> impl Future<Output = Result<ProtocolFlow, TestError>> + MaybeSend {
        async { Ok(ProtocolFlow::Close) }
    }

    fn acquire_channel(
        &self,
        _runtime: &Arc<RuntimeConfig>,
        _outbound: Arc<TestOutbound>,
    ) -> impl Future<Output = Result<NoChannel, TestError>> + MaybeSend {
        async { Err(TestError) }
    }

    fn send(ctx: TestCtx) -> impl Future<Output = Result<TestCtx, TestError>> + MaybeSend {
        async move { Ok(ctx) }
    }

    fn install_channel(_ctx: &mut TestCtx, channel: NoChannel) {
        match channel {}
    }
}

struct TaggedMiddleware(&'static str);

impl AsyncMiddleware<TestCtx> for TaggedMiddleware {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn return_self() -> Self {
        Self("returned")
    }

    fn handle<'a>(&self, context: TestCtx, next: Box<NextFn<TestCtx>>) -> BoxFuture<TestCtx> {
        next(context)
    }
}

fn middleware(name: &'static str) -> Arc<dyn AsyncMiddleware<TestCtx>> {
    Arc::new(TaggedMiddleware(name))
}

fn middleware_names(chain: &AsyncMiddlewareChain<TestCtx>) -> Vec<&'static str> {
    chain
        .iter()
        .map(|middleware| {
            middleware
                .as_any()
                .downcast_ref::<TaggedMiddleware>()
                .expect("test chain contains only tagged middleware")
                .0
        })
        .collect()
}

fn final_handler() -> Arc<dyn AsyncFinalHandler<TestCtx>> {
    Arc::new(|ctx: TestCtx| async move { Ok(ctx) })
}

fn endpoint(url: &str, name: &str) -> Endpoint<TestProto> {
    Endpoint::from_final_handler(url, name, final_handler())
}

impl EndpointOutcome<TestCtx> for TestResponse {
    fn apply_to(self, context: &mut TestCtx) -> Result<(), TestError> {
        context.response = self;
        Ok(())
    }
}

fn unit_body(context: &mut TestCtx) -> MaybeSendBoxFuture<'_, ()> {
    Box::pin(async move {
        core::future::ready(()).await;
        context.response.0 = Some("body-ran");
    })
}

fn response_body(_context: &mut TestCtx) -> MaybeSendBoxFuture<'_, TestResponse> {
    Box::pin(async { TestResponse(Some("response")) })
}

fn result_body(_context: &mut TestCtx) -> MaybeSendBoxFuture<'_, Result<TestResponse, TestError>> {
    Box::pin(async { Ok(TestResponse(Some("result"))) })
}

// `bind` itself is runtime-independent, but `App` carries a runtime type.
// These inert capabilities keep the tests local to hotaru_core.
struct TestRuntime;

#[derive(Debug)]
struct TestRuntimeError;

impl core::fmt::Display for TestRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("inert test runtime")
    }
}

impl core::error::Error for TestRuntimeError {}

struct TestJoin<T>(PhantomData<fn() -> T>);

impl<T: MaybeSend + 'static> Future for TestJoin<T> {
    type Output = Result<T, TestRuntimeError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

struct TestOnceCell<T>(PhantomData<fn() -> T>);

impl<T> Default for TestOnceCell<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: MaybeSendSync + 'static> OnceCellCap<T> for TestOnceCell<T> {
    fn get(&self) -> Option<&T> {
        None
    }

    fn get_or_try_init<'a, F, Fut, E>(&'a self, init: F) -> MaybeSendBoxFuture<'a, Result<&'a T, E>>
    where
        F: FnOnce() -> Fut + MaybeSend + 'a,
        Fut: Future<Output = Result<T, E>> + MaybeSend + 'a,
        E: MaybeSend + 'a,
    {
        drop(init);
        Box::pin(core::future::pending())
    }
}

struct TestMutex<T>(PhantomData<fn() -> T>);
struct TestGuard<'a, T>(PhantomData<&'a mut T>);

impl<T> Deref for TestGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unreachable!("the inert test mutex is never locked")
    }
}

impl<T> DerefMut for TestGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unreachable!("the inert test mutex is never locked")
    }
}

impl<T: MaybeSend + 'static> AsyncMutexCap<T> for TestMutex<T> {
    type Guard<'a>
        = TestGuard<'a, T>
    where
        Self: 'a;

    fn new(_value: T) -> Self {
        Self(PhantomData)
    }

    fn lock(&self) -> impl Future<Output = Self::Guard<'_>> + MaybeSend + '_ {
        core::future::pending()
    }
}

impl RuntimeSpec for TestRuntime {
    type JoinHandle<T: MaybeSend + 'static> = TestJoin<T>;
    type JoinError = TestRuntimeError;

    fn spawn_detached<F>(future: F)
    where
        F: Future<Output = ()> + MaybeSend + 'static,
    {
        drop(future);
    }

    fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + MaybeSend + 'static,
        F::Output: MaybeSend + 'static,
    {
        drop(future);
        TestJoin(PhantomData)
    }

    type Instant = Duration;
    type TimeoutError = TestRuntimeError;

    fn now() -> Self::Instant {
        Duration::ZERO
    }

    fn instant_plus(instant: Self::Instant, duration: Duration) -> Self::Instant {
        instant.saturating_add(duration)
    }

    fn sleep(_duration: Duration) -> impl Future<Output = ()> + MaybeSend + 'static {
        async {}
    }

    fn sleep_until(_deadline: Self::Instant) -> impl Future<Output = ()> + MaybeSend + 'static {
        async {}
    }

    fn timeout<F>(
        _duration: Duration,
        future: F,
    ) -> impl Future<Output = Result<F::Output, Self::TimeoutError>> + MaybeSend
    where
        F: Future + MaybeSend,
        F::Output: MaybeSend,
    {
        async move { Ok(future.await) }
    }

    fn select2<A, B>(a: A, b: B) -> impl Future<Output = Either<A::Output, B::Output>> + MaybeSend
    where
        A: Future + MaybeSend,
        B: Future + MaybeSend,
        A::Output: MaybeSend,
        B::Output: MaybeSend,
    {
        async move {
            let output = a.await;
            drop(b);
            Either::Left(output)
        }
    }

    type OnceCell<T: MaybeSendSync + 'static> = TestOnceCell<T>;
    type AsyncMutex<T: MaybeSend + 'static> = TestMutex<T>;
}

fn server(
    root_middlewares: AsyncMiddlewareChain<TestCtx>,
) -> Arc<Server<TestTransport, TestRuntime>> {
    Arc::new(App::<TestTransport, TestRuntime, InboundOnly> {
        registry: ProtocolRegistryKind::single(
            TestProto,
            Arc::new(UrlRoot::new()),
            root_middlewares,
        ),
        inbound_state: InboundState {
            binding: (),
            inbound: Default::default(),
        },
        outbound_state: (),
        runtime: Arc::new(RuntimeConfig::new()),
        config: OperationalConfig::default(),
        _rt: PhantomData,
        _target: PhantomData,
    })
}

fn client(
    root_middlewares: AsyncMiddlewareChain<TestCtx>,
) -> Arc<Client<TestTransport, TestRuntime>> {
    Arc::new(App::<TestTransport, TestRuntime, OutboundOnly> {
        registry: ProtocolRegistryKind::single(
            TestProto,
            Arc::new(UrlRoot::new()),
            root_middlewares,
        ),
        inbound_state: (),
        outbound_state: OutboundState {
            target: (),
            outbound: Default::default(),
        },
        runtime: Arc::new(RuntimeConfig::new()),
        config: OperationalConfig::default(),
        _rt: PhantomData,
        _target: PhantomData,
    })
}

#[test]
fn constructors_match_and_default_to_inherit() {
    let generic: Endpoint<TestProto> =
        AccessPointDef::new("/x", "route", EndpointHandler::new(final_handler()));
    let endpoint = endpoint("/x", "route");

    assert_eq!(generic.url(), endpoint.url());
    assert_eq!(generic.name(), endpoint.name());
    assert_eq!(generic.url_mode(), endpoint.url_mode());
    assert!(matches!(generic.middlewares(), [MWSlot::Inherit]));
    assert!(matches!(endpoint.middlewares(), [MWSlot::Inherit]));
}

#[tokio::test]
async fn endpoint_applies_unit_outcomes() {
    let def = Endpoint::<TestProto>::endpoint("/unit", "unit", unit_body);

    let context = def
        .handler()
        .final_handler()
        .handle(TestCtx::default())
        .await
        .unwrap();

    assert_eq!(context.response.0, Some("body-ran"));
}

#[tokio::test]
async fn endpoint_applies_protocol_outcomes() {
    let def = Endpoint::<TestProto>::endpoint("/response", "response", response_body);

    let context = def
        .handler()
        .final_handler()
        .handle(TestCtx::default())
        .await
        .unwrap();

    assert_eq!(context.response.0, Some("response"));
}

#[tokio::test]
async fn endpoint_applies_nested_result_outcomes() {
    let def = Endpoint::<TestProto>::endpoint("/result", "result", result_body);

    let context = def
        .handler()
        .final_handler()
        .handle(TestCtx::default())
        .await
        .unwrap();

    assert_eq!(context.response.0, Some("result"));
}

#[tokio::test]
async fn endpoint_accepts_an_already_normalized_final_handler() {
    let def = Endpoint::<TestProto>::from_final_handler(
        "/raw",
        "raw",
        Arc::new(|mut context: TestCtx| async move {
            context.response.0 = Some("raw");
            Ok(context)
        }),
    );

    let context = def
        .handler()
        .final_handler()
        .handle(TestCtx::default())
        .await
        .unwrap();

    assert_eq!(context.response.0, Some("raw"));
}

#[test]
fn middleware_builders_preserve_symbolic_order() {
    let def = endpoint("/x", "route")
        .no_inherit()
        .with_middleware(middleware("route-a"))
        .with_inherit()
        .with_middleware(middleware("route-b"));

    assert!(matches!(
        def.middlewares(),
        [
            MWSlot::Concrete(_),
            MWSlot::Inherit,
            MWSlot::Concrete(_),
        ]
    ));
}

#[test]
fn explicit_middlewares_replace_the_default_chain() {
    let def = endpoint("/x", "route").with_middlewares(Vec::new());

    assert!(def.middlewares().is_empty());
}

#[test]
fn pattern_and_literal_modes_use_their_own_parsers() {
    let pattern = endpoint("/users/<int:id>", "pattern");
    let literal = endpoint("/users/<int:id>", "literal").with_url_mode(UrlMode::Literal);

    let (pattern_path, pattern_names) = pattern.parse_url_pattern().unwrap();
    let (literal_path, literal_names) = literal.parse_url_pattern().unwrap();

    assert!(matches!(pattern_path[2], PathPattern::Regex(_)));
    assert_eq!(pattern_names.index("id"), Some(2));
    assert_eq!(
        literal_path,
        vec![
            PathPattern::literal_path(""),
            PathPattern::literal_path("users"),
            PathPattern::literal_path("<int:id>"),
        ]
    );
    assert!(literal_names.is_empty());
}

#[test]
fn bind_splices_every_inherit_slot_in_place() {
    let app = server(vec![middleware("root-a"), middleware("root-b")]);
    let def = endpoint("/users/<int:id>", "user-detail")
        .no_inherit()
        .with_middleware(middleware("route-a"))
        .with_inherit()
        .with_middleware(middleware("route-b"))
        .with_inherit();

    app.bind(def).unwrap();

    let access_point = app
        .registry
        .entry::<TestProto>()
        .unwrap()
        .access_points
        .get("user-detail")
        .unwrap();
    let node = access_point.resolve().unwrap();
    assert_eq!(node.names().index("id"), Some(2));
    assert_eq!(
        middleware_names(node.binding().middlewares()),
        vec!["route-a", "root-a", "root-b", "route-b", "root-a", "root-b",]
    );
}

#[test]
fn outpoint_body_survives_an_empty_user_chain() {
    let app = client(vec![middleware("root")]);
    let def = Outpoint::<TestProto>::outpoint("/send", "send", middleware("body"))
        .with_url_mode(UrlMode::Literal)
        .with_middlewares(Vec::new());

    app.bind(def).unwrap();

    let node = app
        .registry
        .entry::<TestProto>()
        .unwrap()
        .access_points
        .get("send")
        .unwrap()
        .resolve()
        .unwrap();
    assert!(node.has_handler());
    assert_eq!(middleware_names(node.binding().middlewares()), vec!["body"]);
}

#[test]
fn bind_error_keeps_route_identity() {
    let app = server(Vec::new());

    let error = app
        .bind(endpoint("/users/<int:id", "broken-user"))
        .expect_err("malformed route must fail to bind");

    assert_eq!(error.route_name(), "broken-user");
    assert_eq!(error.route_url(), "/users/<int:id");
    assert_eq!(error.batch_index(), None);
}

#[test]
fn bind_all_stops_at_first_error_and_reports_its_index() {
    let app = server(Vec::new());
    let defs = vec![
        endpoint("/good", "good"),
        endpoint("/broken/<int:id", "broken"),
        endpoint("/never", "never"),
    ];

    let error = app
        .bind_all(defs)
        .expect_err("the malformed middle definition must stop the batch");

    assert_eq!(error.batch_index(), Some(1));
    assert_eq!(error.route_name(), "broken");
    let access_points = &app.registry.entry::<TestProto>().unwrap().access_points;
    assert!(access_points.contains("good"));
    assert!(!access_points.contains("broken"));
    assert!(!access_points.contains("never"));
}

#[tokio::test]
async fn same_path_rebind_is_last_wins_and_refreshes_named_entries() {
    let app = server(Vec::new());
    app.bind(
        endpoint("/same", "first")
            .no_inherit()
            .with_middleware(middleware("old")),
    )
    .unwrap();
    app.bind(
        endpoint("/same", "second")
            .no_inherit()
            .with_middleware(middleware("new")),
    )
    .unwrap();
    app.bind(
        endpoint("/same", "second")
            .no_inherit()
            .with_middleware(middleware("newest")),
    )
    .unwrap();

    let entry = app.registry.entry::<TestProto>().unwrap();
    let first = entry.access_points.get("first").unwrap().resolve().unwrap();
    let second = entry
        .access_points
        .get("second")
        .unwrap()
        .resolve()
        .unwrap();
    let walked = app
        .registry
        .url::<TestProto>()
        .unwrap()
        .walk_str("/same")
        .await
        .unwrap();

    assert!(Arc::ptr_eq(&first, &second));
    assert!(Arc::ptr_eq(&second, &walked));
    assert_eq!(entry.access_points.len(), 2);
    assert_eq!(
        middleware_names(first.binding().middlewares()),
        vec!["newest"]
    );
}
