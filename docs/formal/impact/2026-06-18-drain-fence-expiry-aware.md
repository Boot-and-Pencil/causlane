# Formal Impact Record: drain fence I-007 becomes expiry-aware

## Change metadata

- Change ID: FIR-2026-06-18-drain-fence-expiry-aware
- PR/issue: S03 reference-kernel hardening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-18
- Impact class: F3 (kernel invariant change)

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/drain.rs
crates/causlane-core/src/domain/constraint.rs
crates/causlane-core/src/contract.rs
crates/causlane-codegen/src/kani_target.rs
crates/causlane-replay/src/lib.rs
```

## Summary

The I-007 drain-fence rule had three divergent definitions: the replay oracle
(`LeaseTable::fence_blocked_by_active`) and the Kani-proved bounded rule
(`DrainFenceCheck`) were existence-based (a lease blocks until released), while the
contract `DrainSemantics::can_acquire_fence` was expiry-aware (an expired lease no
longer blocks) — and the `drain.rs` docstring wrongly claimed oracle and contract
were the same rule. This unifies all three on the **expiry-aware** semantics
(`can_acquire_fence`'s documented intent):

> A drain fence over a scope may be acquired only when no lease overlaps that scope
> while still *active* (granted, not released) **and not yet expired** at the
> fence's acquisition time.

Concretely:

- `DrainFenceCheck` gains a per-slot `expired` dimension; `fence_acquirable()` adds
  the `|| expired` conjunct.
- The Kani generator (`kani_target.rs`) is updated so the regenerated harness proves
  the rule over the expiry-extended bounded space (mutation-sensitive).
- The replay oracle routes the decision through the single kernel authority
  `KernelContracts.can_acquire_fence(fence_scope, leases.active_leases(), now)`,
  where `now` is the drain event's `occurred_at` (absent a timestamp it falls back
  to the earliest instant, so nothing is treated as expired — fail-closed).
- The existence-only `LeaseTable::fence_blocked_by_active` is retired in favour of
  an `active_leases()` accessor; the `drain.rs` module docstring is corrected.

It also adds a single-authority documentation note to `ConflictOracle` (I-006):
both the trait methods and `LeaseTable::grant` decide conflicts through the one
shared `claim_modes_conflict` definition (no behavior change).

## Affected invariants

```text
I-007: Drain fences require prior overlapping leases to clear — REFINED to
       expiry-aware: an active overlapping lease blocks a fence only until it is
       released OR expires (whichever comes first).
I-006: unchanged (documentation note only).
new invariant ids: none
```

## Affected formal models

```text
Kani: drain_fence_acquirable_only_without_active_overlap_nondet (regenerated over
      the expiry-extended DrainFenceCheck space — still mutation-sensitive)
Replay: drain_with_active_lease_invalid (non-expired overlap still refuted)
P:    DrainBlocksNewMutableAdmission (models the distinct "drain blocks new
      admission" facet — UNAFFECTED by the fence-acquisition expiry change)
```

## Affected protocols

```text
Drain/fence acquisition: a fence acquisition is evaluated against the active,
non-expired overlapping leases at the fence's acquisition time, not mere lease
existence in the table.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none (the scenario schema
  already carried lease `expires_at` and event `occurred_at`).
- Receipt/coverage fields added/changed/removed: none.
- Core semantic change: I-007 drain-fence acquisition is expiry-aware; the
  existence-only `fence_blocked_by_active` is removed.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| `drain_with_active_lease_invalid` | Replay | `DrainFenceWithActiveOverlap` (non-expired overlapping lease still blocks, now via `can_acquire_fence`) | existing — re-verified through the new routing |
| `drain_fence_acquirable_only_without_active_overlap_nondet` | Kani | `fence_acquirable` matches the expiry-aware formula; dropping a conjunct is a counterexample | required — regenerated |

## Positive evidence (the expired-lease-does-not-block direction)

| Check | Lane | What it proves |
|---|---|---|
| `drain_fence_ignores_expired_leases` (`contract.rs`) | core unit | the exact oracle fn `can_acquire_fence` returns acquirable for an expired overlapping lease and blocked for a non-expired one |
| `fence_blocked_only_by_active_nonexpired_overlap` (`drain.rs`) | core unit | the bounded `DrainFenceCheck` rule, including the expired slot |
| `drain_fence_acquirable_only_without_active_overlap_nondet` | Kani | the rule over the full bounded `(overlaps, active, expired)` space |

A full replay *positive* control (a complete trace that acquires a fence over an
expired lease and is accepted) is structurally precluded: a `RuntimeExecution`
action requires an execution barrier (`MissingRequiredBarrier`), and a barrier
requires non-expired leases (`validate_barrier_leases`), so an expired-lease drain
cannot appear in a complete execution trace. The bounded Kani proof plus the two
unit tests above cover the positive direction instead.

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | `drain_with_active_lease_invalid` | yes | rust |
| Alloy | n/a (does not model I-007) | n/a | rust |
| P | `DrainBlocksNewMutableAdmission` (unaffected) | yes | rust |
| Kani | `drain_fence_acquirable_only_without_active_overlap_nondet` (regenerated) | yes | rust |
| Verus | n/a (not_applicable for I-007) | n/a | proof/all |
| Lean4 | n/a (not_applicable for I-007) | n/a | proof/all |

## Not applicable lanes

Alloy, Verus and Lean4 do not model I-007 (per the coverage matrix). P models the
distinct drain-blocks-new-admission ordering facet, which the fence-acquisition
expiry change does not touch. The rule is owned by the replay oracle plus the Kani
bounded harness.

## Acceptance commands

```bash
just formal-ready
just verification-full
```

Additional commands:

```bash
./tools/cargo-dev test -p causlane-core -p causlane-replay
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
