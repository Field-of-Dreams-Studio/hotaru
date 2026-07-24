use core::any::{TypeId, type_name};
use core::marker::PhantomData;

use crate::app::{Accepts, AppTarget};
use crate::connection::TransportSpec;
use crate::executable::def::{AccessPointDef, FinalHandlerDef};
use crate::executable::middleware::AsyncMiddlewareChain;
use crate::executable::registry::ProtocolEntryRegistry;
use crate::marker::MaybeSendSync;
use crate::prelude::{Arc, ToString, Vec};
use crate::protocol::Protocol;

use super::{BlueprintError, ErasedHomoBlueprint, HomoBlueprint, ProtocolDef, TargetGroups};

/// Reusable, cheaply cloneable set of protocol/flavour groups.
///
/// `AT` is the Blueprint's application-target marker and implements the
/// existing [`AppTarget`] trait. It is not a second target abstraction.
pub struct Blueprint<TS: TransportSpec, AT: AppTarget> {
    pub(crate) inner: Arc<BlueprintInner<TS, AT>>,
}

pub(crate) struct BlueprintInner<TS: TransportSpec, AT: AppTarget> {
    pub(crate) groups: Vec<ErasedHomoBlueprint<TS>>,
    pub(crate) _target: PhantomData<fn() -> AT>,
}

impl<TS: TransportSpec, AT: AppTarget> Clone for Blueprint<TS, AT> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TS: TransportSpec, AT: AppTarget> Default for Blueprint<TS, AT> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(private_bounds)]
impl<TS: TransportSpec, AT: AppTarget> Blueprint<TS, AT> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(BlueprintInner {
                groups: Vec::new(),
                _target: PhantomData,
            }),
        }
    }

    /// Admit a protocol with an empty root-middleware chain.
    pub fn with_protocol<P>(self, protocol: P) -> Result<Self, BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        AT: TargetGroups,
    {
        self.with_protocol_middlewares(protocol, AsyncMiddlewareChain::new())
    }

    /// Admit a protocol with its root-middleware chain.
    ///
    /// Rejects duplicate concrete protocol types. Cloning a Blueprint freezes
    /// its protocol set, while AP admission remains shared between clones.
    pub fn with_protocol_middlewares<P>(
        mut self,
        protocol: P,
        root_middlewares: AsyncMiddlewareChain<P::Context>,
    ) -> Result<Self, BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        AT: TargetGroups,
    {
        let name = protocol.name();
        let inner = Arc::get_mut(&mut self.inner).ok_or(BlueprintError::SharedBlueprint)?;
        if inner
            .groups
            .iter()
            .any(|group| group.protocol_type_id() == TypeId::of::<P>())
        {
            return Err(BlueprintError::DuplicateProtocol(name.to_string()));
        }

        let def = Arc::new(ProtocolDef::new(protocol, root_middlewares));
        inner.groups.extend(AT::make_groups::<TS, P>(def));
        Ok(self)
    }

    /// Call a generated zero-argument constructor exactly once, then retain
    /// its access-point definition.
    pub fn bind<P, H>(
        &self,
        constructor: fn() -> AccessPointDef<P, H>,
    ) -> Result<(), BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        H: FinalHandlerDef<P> + MaybeSendSync,
        AT: Accepts<H>,
    {
        self.insert(constructor())
    }

    /// Retain one already-built access-point definition.
    pub fn insert<P, H>(&self, def: AccessPointDef<P, H>) -> Result<(), BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        H: FinalHandlerDef<P> + MaybeSendSync,
        AT: Accepts<H>,
    {
        let group = self.find_group::<P, H>()?;
        group.access_points.write().defs.push(def);
        Ok(())
    }

    /// Retain a homogeneous batch of access-point definitions.
    pub fn extend<P, H, I>(&self, defs: I) -> Result<(), BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        H: FinalHandlerDef<P> + MaybeSendSync,
        AT: Accepts<H>,
        I: IntoIterator<Item = AccessPointDef<P, H>>,
    {
        let group = self.find_group::<P, H>()?;
        let mut points = group.access_points.write();
        points.defs.extend(defs);
        Ok(())
    }

    fn find_group<P, H>(&self) -> Result<&HomoBlueprint<P, H>, BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        H: FinalHandlerDef<P> + MaybeSendSync,
    {
        self.inner
            .groups
            .iter()
            .find_map(|group| group.as_any().downcast_ref::<HomoBlueprint<P, H>>())
            .ok_or(BlueprintError::ProtocolNotFound(type_name::<P>()))
    }

    pub(crate) fn groups(&self) -> &[ErasedHomoBlueprint<TS>] {
        &self.inner.groups
    }

    pub(crate) fn materialize_registry(&self) -> Result<ProtocolEntryRegistry<TS>, BlueprintError> {
        let mut registry = ProtocolEntryRegistry::new();
        for group in self.inner.groups.iter() {
            group.materialize_into(&mut registry)?;
        }
        Ok(registry)
    }
}

impl<TS: TransportSpec, AT: AppTarget> core::fmt::Debug for Blueprint<TS, AT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut groups = f.debug_map();
        for group in self.inner.groups.iter() {
            groups.entry(
                &group.protocol_name(),
                &format_args!("{} defs via {}", group.len(), group.flavour()),
            );
        }
        groups.finish()
    }
}
