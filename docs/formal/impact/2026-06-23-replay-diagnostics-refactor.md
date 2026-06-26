# Formal Impact Record: replay diagnostics refactor

## Change metadata

- Change ID: FIR-2026-06-23-replay-diagnostics-refactor
- PR/issue: diagnostics debt pass
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (developer/operator diagnostic surface)

## Touched protocol-critical paths

```text
crates/causlane-replay/src/lib.rs
crates/causlane-replay/src/outcome.rs
crates/causlane-replay/src/explain.rs
crates/causlane-replay/tests/error_diagnostics.rs
crates/causlane-replay/tests/explain.rs
```

## Summary

This change is a behavior-preserving replay diagnostics refactor. It introduces a
shared bundle-bound replay outcome helper used by both `ReplayTrace::verify_verdict`
and `ReplayTrace::verify_explain`, so acceptance, stable error metadata, checked
invariants, bundle hash, and trace hash are derived once per diagnostic call.

The public diagnostics surface remains unchanged: `ReplayVerdict`,
`ReplayExplain`, `CausalLocation`, `ReplayError`, and `ReplayErrorCode` remain
available from `causlane_replay`.

## Affected invariants

```text
I-001: unchanged - execution/barrier/capability checks are unchanged.
I-002: unchanged - observed truth ordering checks are unchanged.
I-003: unchanged - projection anchor checks are unchanged.
I-006: unchanged - lease conflict checks are unchanged.
I-007: unchanged - drain fence overlap checks are unchanged.
I-008: unchanged - closed lifecycle terminal checks are unchanged.
I-009: unchanged - witness/authz evidence checks are unchanged.
new invariant ids: none
```

## Affected formal models

```text
none - no formal contour, Formal IR schema, generated model artifact, scenario,
or replay trace schema changes.
```

## Affected protocols

```text
PR-replay-diagnostics: replay acceptance semantics are unchanged. The refactor
only consolidates derived diagnostic metadata construction after the existing
bundle-bound verifier has accepted or rejected a trace.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Public API added/changed/removed: none.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| every `ReplayError` variant | replay integration test | stable code token, invariant mapping, causal location shape | new |
| projection anchor attestation mismatch | replay explain test | I-003, `AnchorAttestationMismatch`, projection and anchor ids | new |
| required witness missing | replay explain test | I-009, `RequiredWitnessMissing`, requirement id | new |
| authz decision missing | replay explain test | I-009, `AuthzDecisionMissing`, authz stage | new |
| bundle hash mismatch | replay explain test | structural `BundleHashMismatch`, no causal location | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | diagnostics integration tests | no | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Formal model and generated-artifact lanes are not regenerated because this
change does not modify protocol semantics, schemas, Formal IR, scenarios, or
generated monitors. The applicable executable lane is replay diagnostics testing.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-replay-diagnostics-changed-files.txt
./tools/formal-exceptions-check
./tools/schema-validate-all
./tools/product-track-status-check
just formal-coverage-matrix-check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
