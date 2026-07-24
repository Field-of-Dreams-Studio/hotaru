use crate::prelude::Arc;
use crate::{
    app::runtime::RuntimeSpec,
    connection::{Inbound, Outbound, TransportSpec},
};

/// Inbound-side state: the bind target plus the once-cell that lazily
/// materialises the built inbound transport.
pub struct InboundState<TS: TransportSpec, Rt: RuntimeSpec> {
    pub binding: <TS::Inbound as Inbound>::BindTarget,
    pub inbound: <Rt as RuntimeSpec>::OnceCell<Arc<TS::Inbound>>,
}

/// Outbound-side state: the connect target plus the once-cell that lazily
/// materialises the built outbound transport.
pub struct OutboundState<TS: TransportSpec, Rt: RuntimeSpec> {
    pub target: <TS::Outbound as Outbound>::ConnectTarget,
    pub outbound: <Rt as RuntimeSpec>::OnceCell<Arc<TS::Outbound>>,
}

/// Marker trait supplying the side-state associated types for each `App`
/// role. Implementors: `InboundOnly`, `OutboundOnly`, `Both`.
///
/// The `'static` bound is sufficient: auto-trait `Send`/`Sync` is
/// derived from the field states of the App, not from the marker.
pub trait AppTarget: 'static {
    type Inbound<TS: TransportSpec, Rt: RuntimeSpec>;
    type Outbound<TS: TransportSpec, Rt: RuntimeSpec>;
}

/// Capability marker for app targets whose inbound state is concrete.
///
/// Inbound-specific `App` methods can use this bound to access
/// `InboundState<TS, Rt>` without being available on outbound-only apps.
pub trait InboundTarget<TS, Rt>: AppTarget<Inbound<TS, Rt> = InboundState<TS, Rt>>
where
    TS: TransportSpec,
    Rt: RuntimeSpec,
{
}

/// Capability marker for app targets whose outbound state is concrete.
///
/// Outbound-specific `App` methods can use this bound to access
/// `OutboundState<TS, Rt>` without being available on inbound-only apps.
pub trait OutboundTarget<TS, Rt>: AppTarget<Outbound<TS, Rt> = OutboundState<TS, Rt>>
where
    TS: TransportSpec,
    Rt: RuntimeSpec,
{
}

/// Server-only role: has `InboundState`, no outbound side.
/// This struct only provide the associated types. Does not carry any data itself.
/// The actual data is stored in the `APP` struct `
pub struct InboundOnly;
/// Client-only role: has `OutboundState`, no inbound side.
/// This struct only provide the associated types. Does not carry any data itself.
/// The actual data is stored in the `APP` struct `
pub struct OutboundOnly;
/// Gateway role: has both sides. `Server::try_combine(Client)` in a
/// future release can yield this by upgrading the target marker.
/// This struct only provide the associated types. Does not carry any data itself.
/// The actual data is stored in the `APP` struct `
pub struct Both;

impl AppTarget for InboundOnly {
    type Inbound<TS: TransportSpec, Rt: RuntimeSpec> = InboundState<TS, Rt>;
    type Outbound<TS: TransportSpec, Rt: RuntimeSpec> = ();
}

impl AppTarget for OutboundOnly {
    type Inbound<TS: TransportSpec, Rt: RuntimeSpec> = ();
    type Outbound<TS: TransportSpec, Rt: RuntimeSpec> = OutboundState<TS, Rt>;
}

impl AppTarget for Both {
    type Inbound<TS: TransportSpec, Rt: RuntimeSpec> = InboundState<TS, Rt>;
    type Outbound<TS: TransportSpec, Rt: RuntimeSpec> = OutboundState<TS, Rt>;
}

impl<TS: TransportSpec, Rt: RuntimeSpec> InboundTarget<TS, Rt> for InboundOnly {}
impl<TS: TransportSpec, Rt: RuntimeSpec> InboundTarget<TS, Rt> for Both {}

impl<TS: TransportSpec, Rt: RuntimeSpec> OutboundTarget<TS, Rt> for OutboundOnly {}
impl<TS: TransportSpec, Rt: RuntimeSpec> OutboundTarget<TS, Rt> for Both {}
