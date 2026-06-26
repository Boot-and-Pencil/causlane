# Formal Impact Record: route capability derivation through KernelContracts

## Change metadata

- Change ID: FIR-2026-06-18-route-capability-via-kernelcontracts
- PR/issue: S03/M03.4 reference-kernel hardening (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-18
- Impact class: F4 (runtime hard-effect path) — behavior-preserving routing refactor

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/guarded_executor.rs
```

## Summary

`GuardedExecutor::spend_barrier` previously derived the execution capability by
calling the free-standing constructor `ExecutionCapability::derive_from_barrier`
directly. It now routes that decision through the canonical kernel authority:

```rust
let capability = KernelContracts
    .derive_capability(barrier, op.index)
    .map_err(SpendError::Capability)?;
```

This advances the S03 goal of making `KernelContracts` the single authority
through which runtime/replay protocol decisions flow, so the formal lanes verify
exactly the surface the runtime spends. It is a pure routing change: no protocol
behavior, contract, IR, or coverage is altered.

## Affected invariants

```text
I-001: Execution requires a prior write-ahead barrier. (binding mechanism only —
       UNCHANGED; the capability is the structural token proving the
       execution↔barrier binding, and its derivation is identical, see below)
I-002:
I-003:
I-004:
I-005:
I-006:
I-007:
I-008:
I-009:
I-010:
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact changes. The capability-binding rule is unchanged,
so the existing Kani harnesses remain the authority for it (see negative controls).
```

## Affected protocols

```text
Barrier-spend / capability-derivation seam: a barrier must be authorized
(deny-by-default) before a scoped capability is derived and spent. Routing the
derivation through KernelContracts does not change when or how the capability is
derived, only the call surface.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core semantic change: none. `KernelContracts::derive_capability`
  (`crates/causlane-core/src/contract.rs:191`) delegates verbatim to
  `ExecutionCapability::derive_from_barrier`
  (`crates/causlane-core/src/domain/capability.rs:83`); the routed call computes
  the identical capability (same `OpNotCovered` / `LeaseCoverageMissing`
  fail-closed errors, same canonical capability id, same lease set and expiry).

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| `capability_binding_rule_is_fail_closed` | Kani | derivation rejects an op/lease not covered by the barrier (`formal/kani/generated/release_promote_success.rs:87`) | existing |
| `capability_binding_is_exact_conjunction_nondet` | Kani | derived capability binds exactly action/plan/op/barrier/lease (`formal/kani/generated/release_promote_success.rs:257`) | existing |
| `barrier_cannot_be_spent_without_authorization` | runtime test | unauthorized barrier returns `Unauthorized`, op never runs (`crates/causlane-runtime/src/guarded_executor.rs:183`) | existing |
| `authorized_barrier_runs_the_op` | runtime test | bound, non-expired allow runs the op exactly once through the routed derivation (`crates/causlane-runtime/src/guarded_executor.rs:202`) | existing |

No new negative control is required: the behavior under test is unchanged, and
the existing controls already exercise both the failure and success paths of the
derivation now reached via `KernelContracts`.

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | n/a (no trace/IR change) | n/a | rust |
| Alloy | n/a (structural binding unchanged) | n/a | rust |
| P | n/a (no protocol interleaving change) | n/a | rust |
| Kani | `capability_binding_rule_is_fail_closed`, `capability_binding_is_exact_conjunction_nondet` (unchanged) | yes | rust |
| Verus | n/a | n/a | proof/all |
| Lean4 | n/a | n/a | proof/all |

## Not applicable lanes

The capability-derivation rule is owned by the Kani harnesses plus the
guarded-executor runtime tests. Alloy/P/Replay model adjacent generated
lifecycle/binding obligations but do not separately model the
`derive_from_barrier` computation; routing it through `KernelContracts` does not
change what those lanes assert. Verus/Lean4 capability facets remain
`non_blocking_spec` outside the proof/all profile and are unaffected.

## Acceptance commands

```bash
just formal-ready
just formal-verify-all
```

Additional commands:

```bash
tools/formal-discipline-check --profile rust --no-diff --json
./tools/cargo-dev test -p causlane-runtime -p causlane-core
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: route the remaining bypass sites (drain/fence I-007,
  lease-conflict I-006, lifecycle structural pass I-001/I-002/I-003) through
  `KernelContracts` as separate FIR-gated increments, since those reroutes are
  behavior-affecting and each needs its own negative control.
