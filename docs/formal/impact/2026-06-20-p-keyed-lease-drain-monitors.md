# Formal Impact Record: keyed P lease/drain monitors (dispatcher-012 P1-001 part 2)

## Change metadata

- Change ID: FIR-2026-06-20-p-keyed-lease-drain-monitors
- PR/issue: dispatcher-012 ТЗ P1-001 (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-20
- Impact class: F3 (formal-model coverage / correctness) — P generator; no
  kernel/invariant semantics change

## Touched protocol-critical paths

```text
crates/causlane-codegen/src/targets.rs
tools/formal-verify-all
contracts/scenarios/lease_during_drain_invalid.scenario.yaml   (new)
docs/formal/dispatcher-012-tz-status.md
```

## Summary

The second P1-001 increment keys the constraint-plane monitors. Previously
`NoConflictingActiveLeases` and `DrainBlocksNewMutableAdmission` used **global
booleans** (`var activeExclusive: bool`, `var draining: bool`), so two exclusive
leases on *different* scopes would false-conflict and a drain over one scope would
block leases on *every* scope — not an interleaving lane.

Two changes:

1. **Per-lease event expansion** (`push_p_machines`): a `constraint.lease_granted` /
   `constraint.lease_released` event is expanded into **one P send per lease**, each
   carrying that lease's `leaseResource` / `leaseScope` / `leaseMode` (new
   `EventPayload` fields). The barrier event's own lease-coverage facts are left as a
   single send (only constraint lease events are expanded), so the lifecycle monitors
   are unaffected.
2. **Keyed monitors:**
   - `NoConflictingActiveLeases` keys `activeExclusive` by `leaseScope` and only
     tracks `exclusive` leases — two active exclusive leases conflict only on the
     same scope (I-006).
   - `DrainBlocksNewMutableAdmission` keys `draining` by scope (drain events carry
     the scope as `factScope`, lease grants as `leaseScope`) — a drain over one scope
     blocks new mutable admissions on that scope only (I-007).

The claim-resolution block was extracted into `p_resolve_claim` to keep
`p_event_payload` under the 100-line clippy cap.

## Non-vacuity proof (anti-theatre)

- The keying is **load-bearing**, proven automatically by the gate: with per-lease
  expansion, `release_promote_success` emits two `ConstraintLeaseGranted` sends on
  *different* scopes (`environment:staging`, `release_candidate:rc_123`) and still
  **passes** the positive P-check (rc=0). A flat boolean would set `activeExclusive`
  on the first send and refute the second — so the success scenario passing is only
  possible *because* the monitor keys by scope.
- `conflicting_leases_invalid` (two exclusive leases on the **same** scope)
  **refutes** in P (rc=1) and still `refuted_by_replay` (`ConflictingLeases`).
- New `lease_during_drain_invalid` (drain fence requested over a scope, then a lease
  granted on that scope) **refutes** in P (rc=1, the keyed drain monitor fires). The
  same invalid trace is `refuted_by_replay` with code `Lifecycle` — the strict
  RuntimeExecution profile rejects a bare `drain.fence_requested` from `New`; the
  declared `expected_error_code` is `Lifecycle` to match what replay actually does
  (the drain-overlap code belongs to the fuller `drain_with_active_lease_invalid`).

## Affected invariants

```text
I-006/I-007: the P lane now checks these per scope (keyed), not via a global boolean.
Single-scope semantics unchanged; the keying fixes multi-scope interleaving
false-positives (concurrent exclusive leases / drains on different scopes).
new invariant ids: none
```

## Affected formal models

```text
P: NoConflictingActiveLeases + DrainBlocksNewMutableAdmission keyed by scope; lease
events expanded one send per lease; EventPayload gains leaseResource/leaseScope/
leaseMode. No Alloy/Kani/Verus/Lean4 change. The success P-check passes; the lease
and drain controls refute.
```

## Contract changes

- Bundle / Formal IR / replay-trace / receipt / coverage fields: none (the new
  `EventPayload` fields are internal to the generated P text).
- Core semantic change: none.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `conflicting_leases_invalid` | P | refute (same-scope exclusive) | added to P gate — verified rc=1 |
| `lease_during_drain_invalid` | P | refute (drain blocks same-scope lease) | new — verified rc=1 |
| `lease_during_drain_invalid` | replay | `Lifecycle` (strict profile) | new — `refuted_by_replay` |
| `release_promote_success` | P | pass (different-scope exclusive) | re-verified rc=0 (keying load-bearing) |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| P | keyed lease/drain monitors + lease/drain controls | generated | rust |

## Not applicable lanes

No Alloy/Kani/Verus/Lean4 change.

## Acceptance commands

```bash
just formal-verify-all
```

## Exception request

- Exception needed? no
- Follow-up (P1-001 remaining): replace the stub
  `Dispatcher`/`LeaseManager`/`Worker` machines + the single sequential
  `ScenarioDriver` with concurrent protocol state machines P actually interleaves
  (the keyed monitors are now ready to verify true interleaving).
