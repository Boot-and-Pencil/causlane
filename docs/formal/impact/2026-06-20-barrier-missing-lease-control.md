# Formal Impact Record: barrier-missing-lease negative control (dispatcher-012 P2-003)

## Change metadata

- Change ID: FIR-2026-06-20-barrier-missing-lease-control
- PR/issue: dispatcher-012 ТЗ follow-ups (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-20
- Impact class: F1 (negative scenario only) — no code/model change; exercises the
  existing replay claim-coverage check

## Touched protocol-critical paths

```text
contracts/scenarios/barrier_missing_lease_invalid.scenario.yaml   (new)
formal/obligations/lifecycle_product_obligations.yaml
docs/formal/dispatcher-012-tz-status.md
```

## Summary

Adds the missing P2-003 negative control that IS replayable: a barrier whose
predicate requires exclusive leases on both `environment:staging` and
`release_candidate:rc_123`, but whose trace grants/carries only the environment
lease. Replay's barrier claim-coverage check
(`validate_claim_manifest_coverage`, `crates/causlane-replay/src/lib.rs:216`)
rejects the uncovered claim with `ReplayError::Lease` (code `Lease`). The control is
auto-collected by `collect_negative_controls` and added to `OBL-I006`
`negative_controls` for discipline adequacy.

The two other P2-003 items are documented as **not replayable** (no code added):
`constraint_update_rewrites_truth` (no constraint-update event kind in the trace
schema; I-010 is a P/Kani concern, replay=not_applicable) and
`projection_anchor_wrong_event_kind` (an anchor at a non-observed-truth event is
already subsumed by `AnchorNotObservedTruth`). P2-001 is documented not-applicable
(its cited dups are intentional — `mergeable()->false` is referenced by the Kani
harness).

## Affected invariants

```text
I-006 (claim coverage): a new negative control proving the barrier claim-coverage
check is non-vacuous. Semantics unchanged.
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact or replay code change; the existing claim-coverage
check is exercised by a new scenario.
```

## Contract changes

- New negative scenario fixture + manifest entry. No bundle/IR/trace/receipt/
  coverage field change. No core/replay code change.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `barrier_missing_lease_invalid` | Replay | `Lease` ("claim not covered") | new — refuted_by_replay |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | `barrier_missing_lease_invalid` (new control) | yes | rust |

## Not applicable lanes

Alloy/P/Kani/Verus/Lean4 unchanged; claim coverage is a replay/runtime check.

## Acceptance commands

```bash
just formal-verify-all
```

## Exception request

- Exception needed? no
- Follow-up: the two not-replayable P2-003 items would only be exercisable if the
  trace schema gained a constraint-update event (I-010) — out of scope.
