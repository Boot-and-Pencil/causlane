# ADR-0005: Tiers, lanes and constraint plane

- Status: accepted
- Date: 2026-06-05

## Context

The project needs flexible concurrency control without hardcoding business resources or resource semantics into the kernel.

## Decision

Use three separate concepts:

```text
Tier
  lifecycle/authority stage.

Lane
  capacity/capability/fairness slot inside a tier.

Constraint Plane
  dynamic/static constraints that may restrict frontier admission.
```

Lanes do not create semantics. They select where already-allowed work runs.

## Consequences

Concurrency can be controlled by resource claims, leases, quotas, freezes, drains and provider snapshots without making the kernel domain-specific.

## Enforcement

- No lane may bypass lifecycle guards.
- Hard constraints block execution barrier.
- Runtime constraint updates are versioned by epoch/snapshot.
- Hard-effect barriers must reference required leases.

## Codification (M05.1)

The tier model is codified in `crates/causlane-core/src/domain/tier.rs` as the
pure, ordered `Tier` enum (`Admission → Planning → Dispatch → Barrier → Execution
→ Observation → Projection → Closure`); each tier names the authority it confers
(`Tier::authority`). Tiers are not a parallel source of truth: `reached_tier`
maps each `LifecycleStage` to the tier it has reached, and an exhaustive test
(`tier_is_monotonic_under_valid_lifecycle_transitions`, over the full
stage × event × profile space) proves every valid `reduce_lifecycle` transition
keeps the reached tier non-decreasing — an action never drops to a lower
authority. Lanes (M05.2) attach within a tier and add no authority.

## Codification (M05.2)

The lane model is codified in `crates/causlane-core/src/domain/lane.rs`: a `Lane`
binds to exactly one `Tier` and carries a concurrency `LaneCapacity`
(`Unbounded` / `Bounded(n)`), an optional provided capability class and a fairness
weight. `lane_admits(lane, active, op_tier, op_requires)` runs the **tier
authority gate first** — a cross-tier action is always `Reject(WrongTier)` — then
capability, then capacity, so a lane confers no semantic authority and cannot
bypass a lifecycle/tier guard. `lane_never_grants_cross_tier_authority` proves
exhaustively over `Tier × Tier` that any `Admit` implies the action was already at
the lane's tier. Fairness *ordering* stays a runtime scheduling concern; the
kernel only carries the weight.

## Codification (M05.3)

The constraint plane's spec/snapshot/decision model is codified in
`crates/causlane-core/src/domain/constraint_spec.rs` and the `ConstraintProvider`
contract in `contract.rs`. A `ConstraintSpec` carries a `ConstraintKind`
(`Freeze` / `TokenBudget{limit}` / `Restrict{note}`); a `ConstraintSnapshot`
carries the epoch, active constraints and active leases a decision is taken
against. `resolve_constraints(snapshot, claims, scopes)` is the first function to
PRODUCE a `ConstraintDecision`, with fail-closed precedence **Deny > Wait >
AllowWithRestrictions > Allow**: a `Freeze` on an exclusive-write scope or a token
claim larger than its budget denies; an active exclusive-lease conflict or a token
claim that would exceed the budget waits; a `Restrict` allows-with-restriction;
otherwise allow with the claims to acquire as leases (at the snapshot epoch). It
reuses the S03 `claim_modes_conflict` primitive and adds token-budget arbitration
(previously absent). `KernelContracts` implements `ConstraintProvider` by
delegating to it; the hexagonal `ConstraintProviderPort` lets domain providers
supply snapshots.

## Codification (M05.4)

The graph-index layer the frontier selector (M05.5) queries is codified in
`crates/causlane-core/src/domain/graph_index.rs`. A `GraphNode` (`OpId`, lane,
required `FactKind`s, written `Scope`s) is derived from an op's effect signature
and lane assignment; a `GraphIndex` keeps four deterministic (`BTreeMap`/
`BTreeSet`) indexes — `wait_by_fact` (ops blocked on an unproduced fact),
`wait_by_scope` (ops blocked because an active op writes their scope),
`active_by_write_scope` (active ops by written scope — the conflict source), and
`ready_by_lane` (structurally-ready ops grouped by lane). "Structurally ready"
means all required facts are produced and no active op writes a conflicting scope;
the constraint-plane (M05.3) and lane-capacity (M05.2) budgets are layered on top
by frontier selection, not here. The index is maintained INCREMENTALLY —
`add_node` / `mark_produced` / `mark_active` / `mark_complete` reclassify only the
affected nodes via a single `recompute` primitive — and `from_state` performs a
full rebuild. The load-bearing executable invariant is **incremental ≡ full
rebuild**: after every incremental event the index equals `rebuilt()` from its own
base state. To use `Scope`/`LaneId`/`ActionId`/`FactKind` as deterministic
`BTreeMap` keys, those newtypes gained `PartialOrd, Ord` (additive). Deterministic
`BTree` iteration (no `HashMap` ordering) keeps the indexes replay/formal-stable.

## Codification (M05.5)

Safe frontier selection is codified in `crates/causlane-core/src/domain/frontier.rs`.
`select_frontier(index, lanes)` consumes the M05.4 `GraphIndex` (via two additive
accessors — `ready_nodes()` and `active_in_lane()`) plus a `BTreeMap<LaneId,
LaneCapacity>` and returns a `FrontierSelection { selected: BTreeSet<OpId>, rejected:
Vec<FrontierRejection> }`. The selected set is a **conflict-free antichain within lane
budget**: (1) no two selected ops write the same `Scope` ("no conflicting mutable
writes"); (2) every selected op is structurally ready — drawn from `ready_by_lane`, so
all its required facts are already produced (the antichain / no-hard-deps property);
(3) per lane, already-active + selected ≤ capacity (reusing M05.2
`LaneCapacity::has_room`); a write scope is an exclusive resource (≤1 selected writer).
`ready_by_lane` already excludes ops conflicting with an *active* writer; the new safety
this adds is the *pending-vs-pending* write conflict between two ready ops. Selection is
greedy in `OpId` order (deterministic for replay/formal parity); each rejected op carries
a `FrontierBlock` reason (`LaneAtCapacity` / `WriteScopeConflict{scope, with}`) — the seed
for M05.8 why-not-parallel. The load-bearing executable property
(`frontier_is_a_conflict_free_antichain_within_budget`) asserts, over an exhausted small
graph/lane-budget space, that the frontier is conflict-free, an antichain, within budget,
and that every rejection has a valid cause (local maximality), plus determinism and a
non-vacuity guard exercising both rejection causes. **Scoped out** (follow-ups): read-write
conflicts (`GraphNode` carries only `writes`; the gate is "mutable writes"); folding the
constraint-plane `resolve_constraints` freeze/token gate into selection (needs per-op
claims, applied upstream at admission; M05.7 wires runtime updates → frontier rebuild);
verified-merge relaxation of the exact-scope conflict.

## Codification (M05.6)

The drain/fence protocol is codified in `crates/causlane-core/src/domain/drain_protocol.rs`,
**on top of** the I-007 single-scope fence-acquisition authority (`drain.rs` /
`DrainSemantics::can_acquire_fence`) without duplicating or altering it (so I-007's existing
Kani/Alloy/P receipts stand). A `DrainTarget` is either `Domain(Scope)` or `Global` (the
domain-vs-global drain); `covers` reuses `ScopeOverlap`. `at_safe_point(target, leases, now,
contracts)` decides when a target's region is quiesced: the `Domain` case **delegates** to
`can_acquire_fence` (one rule, no divergence), and the `Global` case is that rule generalized
to every scope (no lease is active and non-expired anywhere). `drains_independent` decides
when two drains may proceed in parallel — iff their targets are disjoint (a `Global` drain is
never independent), codifying disjoint domains. `op_admissible_during_drain` keeps a read-only
sidecar (`!EffectSignature::is_mutable()`) admissible while the region is frozen and blocks a
mutable op only if it writes into the drained region — the frozen-sidecar rule. Drain epochs
reuse `ConstraintEpoch`: a `DrainRequest { target, epoch }` `governs` an admission only from
its own epoch onward (future-only, consistent with I-010). The load-bearing executable
property (`safe_point_agrees_with_the_i007_authority`) asserts, over the lease
active/expired/overlap space, that `at_safe_point` agrees with the I-007 authority (and the
global generalization), with a non-vacuity guard exercising both safe and unsafe outcomes;
plus unit tests for covers/independence/sidecar/epoch. Pure additive `causlane-core`; no
change to `DrainFenceCheck` / `fence_acquirable` / `can_acquire_fence` / `AuditEventKind` /
codegen, so no bundle-hash or formal-IR regeneration. **Scoped out** (follow-ups): applying a
drain as a runtime `Freeze` + rebuilding the frontier on the epoch bump (M05.7); surfacing the
blocking reason (M05.8); deeper Lean/Verus modeling of the drain-epoch protocol (S10).

## Codification (M05.7)

Runtime constraint updates are codified in
`crates/causlane-core/src/domain/constraint_runtime.rs`, **on top of** the I-010
truth-preservation authority (`constraint_update.rs` /
`ConstraintUpdate::preserves_committed_truth`) without duplicating or altering it. A
`RuntimeUpdateKind` is one of the four kinds — `Capacity{lane, capacity}`,
`Quota{resource, scope, limit}`, `Freeze{scope}`, `RateLimit{resource, scope, max_per_epoch}`.
Every update is applied **at a new epoch** (`next_epoch` = saturating `+1`; `epoch_advances`
asserts strict monotonicity) and is **future-only** (`RuntimeUpdate::governs(admission_epoch)`
holds iff `admission_epoch >= applied_at`, the same future-only semantics as a drain and
consistent with I-010). `apply_to_snapshot(snapshot, kind)` produces a new `ConstraintSnapshot`
at the next epoch: `Quota` upserts the `TokenBudget`, `Freeze` adds the freeze idempotently,
`Capacity` (lane-side) and `RateLimit` leave the snapshot constraints unchanged; stale-epoch
leases are dropped (`lease_current(lease, epoch)` — a lease is valid only within its grant
epoch, ADR-0005/0013). **No truth rewrite** reuses the I-010 authority: `truth_rewrite_of(kind)`
returns an empty rewrite mask, so a runtime update preserves committed truth for *any* committed
state. **Frontier rebuild** reuses M05.4/M05.5: `apply_capacity(lanes, kind)` produces the
updated lane registry that `select_frontier` is re-run against (an integration test shows a
capacity tightening rebuild a smaller frontier). The load-bearing property
(`apply_preserves_truth_bumps_epoch_and_drops_stale_leases`) asserts, across all four kinds,
strict epoch advance + committed-truth preservation (over every committed state) + stale-lease
drop, with a non-vacuity guard that constraint-resident kinds change the constraints while
lane-side/rate-limit kinds do not. Pure additive `causlane-core`; no change to `CommittedTruth`
/ `ConstraintUpdate` / `preserves_committed_truth` / `AuditEventKind` / codegen, so the I-010
Kani/Verus/P receipts stand with no regeneration. **Scoped out** (follow-ups): `RateLimit`
*enforcement* (no constraint-plane primitive yet — the kind exists and bumps the epoch but is
not wired into `resolve_constraints`); unfreeze (Freeze is add-only here); surfacing the
blocking reason (M05.8); deeper Lean/Verus modeling of the epoch-bump theorem L-013 (S10).

## Codification (M05.8)

why-not-parallel — the machine-readable blocker/rationale that closes the S05 exit gate
(a dispatcher can explain `ready`, `blocked`, and *why-not-parallel*) — is codified in
`crates/causlane-core/src/domain/why_not_parallel.rs`. It is an **aggregator**: a single
`NotParallelReason` vocabulary whose every variant *wraps an existing typed cause by value*
and re-derives nothing — `Frontier(FrontierBlock)` (M05.5), `ConstraintWait(ConstraintBlocker)`
/ `ConstraintDeny(ConstraintViolation)` (M05.3/M05.7), `LaneRejected(LaneRejection)` (M05.2),
`DrainRegion{target, scope}` (M05.6), and the M05.4 index causes `BlockedOnFact{fact}` /
`BlockedOnActiveScope{scope, held_by}`. `why_not_parallel(op_id, &WhyNotParallelInputs)` unions
the caller-supplied outputs of `select_frontier` / `resolve_constraints` /
`op_admissible_during_drain` / the graph-index queries in a fixed order (deterministic for
replay); `Allow` / `AllowWithRestrictions` contribute nothing (a restriction is not a bar to
parallelism). `WhyNotParallel::is_parallelizable()` is true iff no cause was aggregated;
`reason_from_frontier` lifts a single rejection; `pair_conflict` answers the pairwise case
(only the TD-010 shared-write-scope bar — lane budget is per-lane, surfaced per-op). The
load-bearing property (`aggregation_is_faithful_and_sound`) asserts, across the product of all
input categories, that the answer is **value-faithful** (every supplied cause appears as
exactly one reason carrying its values — nothing invented or dropped) and **sound**
(`is_parallelizable()` ⟺ zero causes — a rejected op is never reported runnable, a clear op
never blocked), with a non-vacuity guard exercising all seven reason variants. "Machine-readable"
for the Rust-first kernel is the typed enum; `causlane-core` has no `serde`, so a JSON surface is
a downstream concern (the replay/CLI crates, like `ReplayExplain`) — out of scope here. Pure
additive; no change to any formal-bound type or codegen → no bundle-hash or formal-IR
regeneration. With M05.8 every S05 milestone (M05.1–M05.8) is codified and the exit gate is met.
