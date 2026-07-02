# Formal Impact Record: legacy witness reconciliation (dispatcher-012 P0-006)

## Change metadata

- Change ID: FIR-2026-06-19-legacy-witness-reconciliation
- PR/issue: dispatcher-012 ТЗ, DoD fix stage (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F4 (runtime/replay evidence path) — tightens I-009 witness handling;
  no invariant semantics change

## Touched protocol-critical paths

```text
crates/causlane-replay/src/lib.rs
crates/causlane-replay/src/error.rs
contracts/scenarios/barrier_legacy_witness_extra_invalid.scenario.yaml   (new)
verification/formal-full/obligations/lifecycle_product_obligations.yaml
```

## Summary

An `execution.barrier_logged` event carried witness evidence in two places: the
deprecated legacy `AuditEvent.witnesses` (a flat `Vec<AuditEventId>`) and the
authoritative typed `ExecutionBarrier.witnesses` payload (`Vec<WitnessRef>` with
binding/scope/fact). Replay validated the typed payload fully
(`validate_typed_witnesses`) **and** ran a weaker legacy count/prior check
(`validate_barrier_witnesses`) over the legacy list. The two could silently
disagree, so a trace could present one story in the legacy field and another in the
typed payload.

This makes the typed payload the single authority and reconciles the legacy field
against it:

- `validate_barrier_witnesses` (legacy count/prior) is removed — the typed path
  already enforces prior-ness, required-witness presence, binding, scope and
  producer attestation.
- New `validate_legacy_witness_consistency(barrier, barrier_payload)`: when the
  legacy `AuditEvent.witnesses` list is non-empty it must name **exactly** the
  producer events of the typed witnesses, else replay rejects with the new stable
  code `LegacyWitnessMismatch`. An empty legacy list is the canonical typed-only
  path. Legacy witnesses are never sufficient evidence on their own.

## Affected invariants

```text
I-009: Witness/authz evidence must bind exact action/plan/impact/scope — the typed
       ExecutionBarrier.witnesses payload is now the sole evidence authority; the
       legacy AuditEvent.witnesses field is a reconciled compatibility mirror.
       Semantics unchanged for well-formed traces (the success path is identical);
       this closes a legacy/typed divergence hole.
new invariant ids: none
```

## Affected formal models

```text
Replay: new ReplayError::LegacyWitnessMismatch (stable code LegacyWitnessMismatch).
No generated artifact (Alloy/P/Kani/Verus/Lean4) change; no regeneration.
```

## Contract changes

- New replay error variant + stable code `LegacyWitnessMismatch`.
- Bundle / Formal IR / trace schema / receipt / coverage fields: none.
- Core semantic change: none for well-formed traces; rejects legacy/typed
  divergence that was previously unchecked.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `barrier_legacy_witness_extra_invalid` | Replay | `LegacyWitnessMismatch` (legacy list carries an extra event id absent from the typed payload) | new — refuted_by_replay (auto-collected) |
| `missing_witness_invalid` | Replay | `RequiredWitnessMissing` | existing — re-verified via the typed path |
| `release_promote_success` | Replay | pass (legacy `[evt_readiness_ok]` mirrors the typed witness) | existing — re-verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | `barrier_legacy_witness_extra_invalid` (new), `missing_witness_invalid` (unchanged) | yes | rust |
| Alloy/P/Kani | I-009 bindings unchanged | yes | rust |

## Not applicable lanes

No generated-model change; the reconciliation is replay-only (the typed witness
binding the formal lanes model is unchanged).

## Acceptance commands

```bash
just verification-full
./tools/cargo-dev test -p causlane-replay
```

## Exception request

- Exception needed? no
- Follow-up issue: the legacy `AuditEvent.witnesses` field should eventually be
  removed from the protocol-critical path entirely (it is now strictly a reconciled
  mirror); tracked for a later schema-deprecation increment.
