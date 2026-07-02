# Formal Impact Record: closed terminal reducer guard

## Change metadata

- Change ID: FIR-2026-06-18-closed-terminal-reducer
- PR/issue: local formal evidence refresh
- Owner: repo maintainers
- Date: 2026-06-18
- Impact class: F3

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/lifecycle.rs
crates/causlane-codegen/src/kani_target.rs
scripts/check-verification-full.sh
docs/invariants/**
```

## Affected invariants

```text
I-008: Closed is terminal.
```

## Affected formal models

```text
Kani: closed_stage_is_terminal_nondet
Replay: event_after_closed_invalid
Verus: closed terminality preservation proof
Lean4: closed_is_terminal theorem application
```

## Affected protocols

```text
Lifecycle reducer protocol: Closed must reject all subsequent events, including
observational RuntimeExecution events such as GateDenied and ViolationDetected.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Core semantic change: `reduce_lifecycle(Closed, *, *)` is fail-closed.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| `event_after_closed_invalid` | Replay | `EventAfterClosed` | existing |
| `closed_stage_is_terminal_nondet` | Kani | no accepted event from `Closed` | required |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Replay | `event_after_closed_invalid` | yes | rust |
| Kani | `closed_stage_is_terminal_nondet` | yes | rust |
| Verus | closed terminality lemmas | yes | proof/all |
| Lean4 | `closed_is_terminal` | yes | proof/all |

## Not applicable lanes

Alloy and P cover adjacent generated lifecycle/event obligations in the current
rust profile, but this reducer-specific regression is owned by the Kani harness
plus the replay negative control.

## Acceptance commands

```bash
scripts/check-verification-full.sh --profile rust --lane local_smoke
tools/formal-discipline-check --profile rust --no-diff --json
```

Additional commands:

```bash
cargo test -p causlane-core lifecycle_closed_is_terminal_and_new_is_initial
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
