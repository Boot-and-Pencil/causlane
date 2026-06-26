# Formal Impact Record: sync DEMO_PLAN_HASH pin + fix examples docstring

## Change metadata

- Change ID: FIR-2026-06-18-demo-plan-hash-pin-sync
- PR/issue: pre-existing debt cleanup (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-18
- Impact class: F1 (tests/scenarios only) — golden-value sync + docstring honesty fix

## Touched protocol-critical paths

```text
crates/causlane-contracts/src/tests.rs
crates/causlane-contracts/src/examples.rs
```

## Summary

The pinned `DEMO_PLAN_HASH` canary in `tests.rs` had drifted from the compiled
demo bundle: commit `94623cd` ("Harden formal coverage obligation mapping") added
a sixth `scenario_ref` to `contracts/examples/release_promote.registry.yaml`,
which legitimately changed the bundle hash and therefore the bundle-bound demo
plan hash, but the pin was not updated. This synchronises the pin to the current
computed value and corrects a stale docstring in `examples.rs` that incorrectly
claimed the hand-authored `release_promote.trace.json` plan hash equals the
bundle-derived material (it does not, and no test enforces equality).

This is the canary working as intended — it caught a real bundle change. There is
no change to the plan-hash computation, the bundle schema, any invariant, or any
formal artifact.

## Affected invariants

```text
none — the plan-hash binding rule (plan binds to the compiled bundle hash) is
UNCHANGED. Only the recorded golden value moves with the legitimately-changed
bundle. I-005 (routes/profiles cannot drift from the compiled bundle) is honored,
not weakened: the bundle binding is exactly what shifted the pin.
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact, receipt, or coverage change.
```

## Contract changes

- Bundle fields added/changed/removed: none (the registry change that shifted the
  hash landed earlier in `94623cd`; this record only re-pins the test golden value).
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| `plan_hash_pins_to_demo_value` | contracts unit test | demo plan hash equals the pinned bundle-bound value | existing (re-synced) |
| `plan_hash_is_sensitive_to_ops` | contracts unit test | mutating ops changes the plan hash | existing |

No new control required: the existing canary is what detected the drift.

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | n/a | n/a | rust |
| Alloy | n/a | n/a | rust |
| P | n/a | n/a | rust |
| Kani | n/a | n/a | rust |
| Verus | n/a | n/a | proof/all |
| Lean4 | n/a | n/a | proof/all |

## Not applicable lanes

No formal lane models the demo's pinned golden value; it is a contracts-crate unit
canary over the canonical plan-hash material. The plan-hash computation itself is
unchanged, so no lane's assertions move.

## Acceptance commands

```bash
just formal-ready
just formal-verify-all
```

Additional commands:

```bash
./tools/cargo-dev test -p causlane-contracts plan_hash_pins_to_demo_value
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
