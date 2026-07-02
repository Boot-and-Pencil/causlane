# Formal readiness final report

> **Superseded (2026-06-08).** This report is a historical snapshot; its hashes
> and "partial" rows predate the P0-004/005/006, Formal IR v2, three-lane
> attestation, and authz policy/freshness/formal-lane work. See
> [`formal-readiness-status.md`](formal-readiness-status.md) for the current,
> machine-derived status; the live authority is `just verification-full`.

Status date: 2026-06-05.

`just formal-ready` is the readiness gate for the current repository state. It
passes the full chain:

```text
formal doctor
Rust fmt/check/test
registry -> compiled bundle
bundle-bound replay over checked trace
scenario -> emitted trace -> bundle-bound replay
bundle+scenario -> generated Alloy facts -> receipt v2
stale-check over generated facts + receipt
formal smoke over Alloy core + generated success/negative controls
```

Current demo hashes:

```text
bundle_hash: sha256:1cda1117cc0e6cc9c4e80101fdd6c8354d0f0f34f7410ffaf623b7b7c99c2faf
plan_hash:   sha256:a241276dc389e9197710cd415072e850036429799d636873f47ae8a1bc44d47b
```

The machine-readable run report is written to
`target/causlane/formal-readiness-report.json`.

## Closed in final hardening

| FFR item | Status |
|---|---|
| Canonical Serialization v1 | done: `canonical_json_bytes/hash`, golden test, BundleBody v3 hashes |
| BundleBody v3 / formal input v0.3 | done: authz selector, witness schema, claim templates, lease/impact/lifecycle/projection policy, scenario refs |
| Exact template resolver | done: typed subject/circumstance bindings, exact witness/claim resolution |
| Replay v0.2 | done: stable error-code API, bundle-bound barrier/witness/lease/authz/capability checks |
| Authz evidence model | done: typed `authz.decision_recorded`, allow/deny/binding checks for required policies |
| Execution capability integrity | done: `ExecutionCapability::derive_from_barrier`, execution-start capability validation |
| Scenario catalog v0.2 | partial but executable: success plus negative replay controls with expected error codes |
| Alloy facts v0.2 | partial: generated facts include formal obligations, required witness/claim/authz/scenario-ref sets, expected result |
| Formal receipt schema v2 | done: codegen receipt binds bundle/scenario/artifact/generator/invariants |
| Formal-ready CI gate | done: `tools/formal-ready`, `just formal-ready` |

## Deferred to FM

These are no longer P0 readiness blockers, but they are the first formal-model
implementation work:

```text
FM-001 Alloy bundle-bound structural model beyond MVP checks
FM-002 P protocol/interleaving model
FM-003 Kani bounded Rust checks
FM-004 Verus abstract preservation proofs
```

The authority surface remains the compiled bundle plus generated facts,
receipts and stale-check. Generic/manual formal sketches are not authoritative
by themselves.
