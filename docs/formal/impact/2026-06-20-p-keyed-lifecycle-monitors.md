# Formal Impact Record: keyed P lifecycle monitors (dispatcher-012 P1-001 part 1)

## Change metadata

- Change ID: FIR-2026-06-20-p-keyed-lifecycle-monitors
- PR/issue: dispatcher-012 ТЗ P1-001 (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-20
- Impact class: F3 (formal-model coverage / correctness) — P generator; no
  kernel/invariant semantics change

## Touched protocol-critical paths

```text
crates/causlane-codegen/src/targets.rs
tools/formal-verify-all
docs/formal/dispatcher-012-tz-status.md
```

## Summary

The generated P model's lifecycle monitors used **global booleans**
(`var barrierSeen: bool`, `var closed: bool`, …), so under interleaving a barrier /
close / deny for one action incorrectly affected another action — a "sequence
smoke", not an interleaving lane (the P1-001 complaint). The payload-bound monitors
(`CapabilityBindsToBarrier`, `WitnessFactGrounded`, `AnchorFactGrounded`,
`AuthzDecisionGroundsBarrier`) were already keyed; this is the first P1-001
increment, keying the lifecycle family.

Nine monitors are now keyed by `actionId` (`map[string, bool]`), so each tracks
per-action state and stays correct under interleaving:

- `NoExecutionBeforeBarrier` (I-001), `NoObservedWithoutExecution` (I-002),
  `NoProjectionWithoutAnchor` (I-003), `NoEventsAfterClosed` (I-008),
  `ReplayAcceptsOnlyValidTrace`, `ConstraintUpdateDoesNotRewriteTruth` (I-010),
  `AuthzRevocationBeforeBarrierBlocksExecution`, `ApprovalBindingDoesNotDrift`
  (I-009 coarse), `NoDuplicateHardExecutionForSameIdempotencyKey`.
- The redundant I-008 clauses in the ordering monitors were dropped (owned by the
  keyed `NoEventsAfterClosed`), collapsing the Open/Closed state pairs into a single
  keyed state.

The monitor names are unchanged, so `obligations.rs`/`present_obligations` and the
coverage matrix are unaffected.

## Non-vacuity proof (anti-theatre)

- `p compile` + `p check` of `release_promote_success` **passes** with the keyed
  monitors (rc=0).
- Four new lifecycle P negative controls are wired into `tools/formal-verify-all`
  and each **refutes** (the keyed monitor fires, rc=1):
  `execution_without_barrier_invalid` (I-001), `observed_without_execution_invalid`
  (I-002), `projection_without_anchor_invalid` (I-003),
  `event_after_closed_invalid` (I-008). The existing payload-bound controls still
  refute.

## Affected invariants

```text
I-001/I-002/I-003/I-008/I-009/I-010: the P lane now checks these per action (keyed),
not via a global boolean. Semantics unchanged for single-action traces; the keying
fixes multi-action interleaving false-positives/negatives.
new invariant ids: none
```

## Affected formal models

```text
P: 9 lifecycle monitors keyed by action. No Alloy/Kani/Verus/Lean4 change. The
success scenario P-check passes; the 4 new lifecycle controls refute.
```

## Contract changes

- Bundle / Formal IR / replay-trace / receipt / coverage fields: none.
- Core semantic change: none.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `execution_without_barrier_invalid` | P | refute (NoExecutionBeforeBarrier keyed) | new — verified rc=1 |
| `observed_without_execution_invalid` | P | refute | new — verified |
| `projection_without_anchor_invalid` | P | refute | new — verified |
| `event_after_closed_invalid` | P | refute | new — verified |
| existing payload-bound P controls | P | still refute | re-verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| P | keyed lifecycle monitors + 4 lifecycle controls | generated | rust |

## Not applicable lanes

No Alloy/Kani/Verus/Lean4 change.

## Acceptance commands

```bash
just formal-verify-all
```

## Exception request

- Exception needed? no
- Follow-up (P1-001 remaining): key the lease/drain monitors by resource/scope
  (needs an `EventPayload` extension with lease resource/scope/mode); replace the
  stub `Dispatcher`/`LeaseManager`/`Worker` machines + single `ScenarioDriver` with
  concurrent protocol state machines P actually interleaves.
