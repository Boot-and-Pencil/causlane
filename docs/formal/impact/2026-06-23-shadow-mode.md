# Formal Impact Record: Shadow mode diagnostics (M08.8)

## Change metadata

- Change ID: FIR-2026-06-23-shadow-mode
- PR/issue: S08 / M08.8 Shadow mode
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime diagnostics surface)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/shadow.rs
crates/causlane-runtime/src/in_process/tests.rs
crates/causlane-runtime/src/lib.rs
docs/product-track/09-runtime-adapter-track.md
docs/product-track/milestones/m08.8-shadow-mode.md
```

## Summary

M08.8 adds an additive, feature-gated shadow comparison API for
`InProcessRuntimeEvent`. Host integrations collect runtime events through the
existing subscription stream, provide `ShadowExpectation` values, and receive a
`ShadowComparison` describing matched, mismatched and unexpected observations.

The comparison is diagnostic-only. It cannot admit, schedule, retry, cancel,
block, or execute runtime work.

## Affected invariants

```text
I-001: unchanged - capability/authz enforcement remains in existing runtime and
       guarded executor paths.
I-002: unchanged - observed truth semantics and replay ordering are unchanged.
I-003: unchanged - projection anchor semantics are unchanged.
ADR-0011: unchanged - authz remains deny-by-default before execution.
ADR-0013 / M06.6: unchanged - hard effects still require spend-time admission.
new invariant ids: none.
```

## Affected formal models

```text
FM-013 Runtime adapter/port compliance model: strengthened by a diagnostic
shadow observer over in-process runtime events.
No generated Alloy/P/Kani/Verus/Lean artifacts are changed.
```

## Affected protocols

```text
PR-runtime-adapter-boundary: preserved - shadow comparison observes adapter
events but does not create semantic authority.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Runtime public API changed: additive `tokio-runtime` exports for
  `ShadowExpectation`, `ShadowExpectationKind`, `ShadowObservation`,
  `ShadowComparison`, `ShadowMismatch`, `ShadowStatus`, and
  `compare_shadow_events`.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| missing expected event | runtime unit | `ShadowStatus::Mismatch` with missing actual | new |
| extra runtime event | runtime unit | unexpected observation reported | new |
| payload mismatch | runtime unit | actual observation attached to mismatch | new |
| unkeyed rejection | runtime unit | cannot satisfy keyed expectation | new |
| wrong expectation after execution | runtime integration | task still executes; comparison reports mismatch only | new |

## Deferred controls

| Scenario | Deferred to | Reason |
|---|---|---|
| production migration rollout playbook | M12.3 | M08.8 only ships the bounded comparison API |
| runtime enforcement from shadow mismatch | not planned for M08.8 | shadow mode is observation-only by design |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Unit (runtime) | Shadow comparer tests | no | rust |
| Integration (runtime) | In-process event-stream shadow tests | no | rust |
| Product docs | M08.8 runtime shadow docs | no | rust |
| Replay | unchanged | yes | rust |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Replay and generated formal lanes are unchanged because M08.8 adds a runtime
diagnostic view over already-emitted events. It does not add lifecycle states,
truth records, replay schemas, durable payloads, or enforcement semantics.

## Acceptance commands

```bash
./tools/cargo-dev check -p causlane-runtime --all-targets --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --features tokio-runtime --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --features tokio-runtime --locked
./tools/cargo-dev check -p causlane-runtime --all-targets --all-features --locked
./tools/cargo-dev test -p causlane-runtime --all-targets --all-features --locked
./tools/cargo-dev fmt --all --check
git diff --check
just check
just clippy
just test
just coverage-clean && just coverage
./tools/schema-validate-all
./tools/product-track-status-check
./tools/formal-exceptions-check
just formal-coverage-matrix-check
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m08.8-changed-files.txt
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M12.3 should document adoption playbooks and reference
  integration migration flows that consume this diagnostics API.
