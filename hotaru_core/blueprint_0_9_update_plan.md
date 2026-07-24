# Blueprint / AppTarget — 0.9 Update Plan

Status: **planning only.** Nothing here ships in 0.8. This document is the
single owner of the follow-up work deferred out of the 0.8 Blueprint stages.

## Why this plan exists

0.8 keeps the intended long-term Blueprint axis:

```rust
App<TS, Rt, T: AppTarget>
Blueprint<TS, T: AppTarget>
ConfiguredBlueprint<TS, T: AppTarget>
```

To make that axis work in 0.8 without a larger redesign, 0.8 ships one
deliberately temporary piece: the crate-private
`hotaru_core::app::blueprint::target::TargetGroups` bridge. It turns an
`AppTarget` marker into the concrete homogeneous storage groups a Blueprint
holds (`InboundOnly -> endpoint`, `OutboundOnly -> outpoint`,
`Both -> both`).

That bridge is fine as short-lived plumbing, but it restates the role/flavour
table that `Accepts<H>` already owns at bind time. 0.9 removes that
duplication and cleans up the surrounding target model.

## 0.8 baseline (do not change in 0.8)

- `AppTarget` is GAT-only: `Inbound<TS, Rt>` / `Outbound<TS, Rt>` associated
  types plus the `InboundTarget` / `OutboundTarget` capability marker traits.
  It carries **no** `HAS_INBOUND` / `HAS_OUTBOUND` capability constants.
- `Blueprint` / `ConfiguredBlueprint` are parameterized by `T: AppTarget`.
- `TargetGroups` is crate-private and marked TEMPORARY in
  `blueprint/target.rs`.
- Public app roles are `Server` (`InboundOnly`) and `Client` (`OutboundOnly`).
  `Both` / `Gateway` remain type-reserved; construction stays deferred.
- `Accepts<H>` remains the per-call `bind` / `insert` / `extend` gate.

## Goals for 0.9

1. Remove the temporary `TargetGroups` role/flavour duplication.
2. Make `AppTarget` a real, documented, public design concept.
3. Keep the `Blueprint<TS, T: AppTarget>` conceptual axis; avoid any
   type-parameter migration for `Blueprint` / `ConfiguredBlueprint`.
4. Land `Both` / `Gateway` construction on top of the cleaned-up model.

## Non-goals for 0.9

- No change to the `AccessPointDef<P, H>` / `Endpoint<P>` / `Outpoint<P>`
  flavour types.
- No switch of `Blueprint` to a flavour parameter (`Blueprint<TS, F>`); that
  path was considered and rejected because it cannot express a single mixed
  `Both` Blueprint and points the type model away from `AppTarget`.
- No change to the `Accepts<H>` bind gate contract.

## Proposed 0.9 direction

Pick one of the two options below; both delete the standalone `TargetGroups`
trait in `blueprint/target.rs`.

### Option A — capability constants on `AppTarget` (single source of truth)

Reintroduce capability metadata on `AppTarget` itself and derive storage
groups from it with one blanket impl, so there is no second table:

```rust
pub trait AppTarget: 'static {
    type Inbound<TS: TransportSpec, Rt: RuntimeSpec>;
    type Outbound<TS: TransportSpec, Rt: RuntimeSpec>;
    const HAS_INBOUND: bool;
    const HAS_OUTBOUND: bool;
}

// Blueprint-side, derived — not a hand-written per-target table.
fn make_groups<T: AppTarget, TS, P>(def: Arc<ProtocolDef<P>>)
    -> Vec<ErasedHomoBlueprint<TS>>
{
    let mut groups = Vec::new();
    if T::HAS_INBOUND  { groups.push(endpoint_group::<P, TS>(def.clone())); }
    if T::HAS_OUTBOUND { groups.push(outpoint_group::<P, TS>(def)); }
    groups
}
```

Trade-off: the const-branch form drops the compile-time impossibility that
`InboundOnly` can construct an outpoint group; correctness then funnels through
`AppTarget`'s own impls instead. This is the minimal-change option.

### Option B — broker layer (richer routing)

Introduce a small broker that owns target-capability routing and group
creation, so both `bind` admission and Blueprint materialization consult one
authority. This is the larger option; prefer it only if 0.9 also needs richer
multi-role / gateway routing than a pair of capability bits can express.

**Recommendation:** start with Option A (smallest change that removes the
duplication); escalate to Option B only if gateway/broker requirements demand
it.

## `AppTarget` publicization

Today `AppTarget` is reachable at `hotaru_core::app::AppTarget` but is **not**
surfaced through the `hotaru` umbrella or prelude, so umbrella users treat it
as internal. 0.9 should:

- surface `AppTarget` (and the role markers) through the umbrella facade and
  prelude;
- document it as stable public API;
- commit to whichever capability surface Option A/B chooses as part of that
  public contract.

## `Both` / Gateway construction

With the target model cleaned up and public, 0.9 can add real `Both` /
`Gateway` construction:

- a single mixed Blueprint (`Blueprint<TS, Both>`) materializing both flavour
  groups, and/or
- a `Server + Client -> Gateway` combine that upgrades the target marker.

Until then, a future `Both` app can still apply an inbound Blueprint and an
outbound Blueprint separately.

The 0.8 `AppBuilder` deliberately uses one shared
`operation_timeout: Option<TimeoutSetting>` because each builder constructs
exactly one role: Server maps it to its frame-processing timeout, while Client
maps it to its request timeout. A real dual-role `Both` builder may need those
two values simultaneously. When Gateway construction lands, decide whether
the broker/target model supplies separate inbound and outbound operation
timeouts; do not force the 0.8 shared builder slot to represent both at once.

## Migration / compatibility notes

- Because the conceptual axis stays `Blueprint<TS, T: AppTarget>`, 0.9 does not
  force a `Blueprint` type-parameter migration on downstream code.
- Removing the standalone `TargetGroups` trait is crate-private and invisible
  to downstream users.
- Making `AppTarget` public is additive; adding capability constants (Option A)
  is a breaking change **only** for any downstream `impl AppTarget for ...`,
  which is not supported/public in 0.8, so the 0.8→0.9 break is contained.

## Checklist for 0.9

- [ ] Delete the TEMPORARY `TargetGroups` trait/impls in
      `blueprint/target.rs`.
- [ ] Land Option A (or B) as the single source of truth for storage groups.
- [ ] Keep `Blueprint` / `ConfiguredBlueprint` on `T: AppTarget`.
- [ ] Surface and document `AppTarget` as public API (umbrella + prelude).
- [ ] Add `Both` / `Gateway` construction and tests.
- [ ] Verify `cargo test -p hotaru_core` and the no_std host build stay green.
