# Formal Impact Record: route ingress reservation

## Change metadata

- Change ID: FIR-2026-06-25-route-ingress-reservation
- PR/issue: M1 runtime partition finding
- Owner: repo maintainers
- Date: 2026-06-25
- Impact class: F2 (runtime adapter admission behavior, no schema or model change)

## Touched protocol-critical paths

```text
crates/causlane-runtime/src/in_process/mod.rs
crates/causlane-runtime/src/in_process/coordinator.rs
crates/causlane-runtime/src/in_process/tests/recovery.rs
docs/adr/0017-host-dispatch-api-v2-partition-coordinator.md
docs/adr/0019-in-process-backpressure-policy.md
docs/refactor/code-review-finding-resolution-matrix-2026-06-25.md
docs/release/publication-blockers-dispatcher-020.md
```

## Summary

Wait-mode routed submit now reserves primary ingress capacity before it acquires
route permits. A routed submit waiting for capacity in a saturated primary
partition therefore does not hold participant permits and does not block
independent participant admissions.

Fail-fast routed submit keeps its immediate route-permit acquisition and
`try_send` behavior. The runtime still coordinates admission only; it does not
become a durable scheduler or semantic authority.

## Affected formal models

```text
none - no bundle, Formal IR, replay trace, generated model, schema, receipt or
coverage-matrix field changes. In-process runtime admission events remain
adapter diagnostics, not replay/formal inputs.
```

## Contract changes

- Bundle fields added/changed/removed: none.
- Formal IR fields added/changed/removed: none.
- Replay trace/scenario fields added/changed/removed: none.
- Receipt/coverage fields added/changed/removed: none.
- Runtime public API changed: none.
- Runtime behavior changed: wait-mode primary ingress reservation now precedes
  route permit acquisition.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| Primary ingress saturated during wait-mode routed submit | runtime unit | independent participant `try_submit` succeeds instead of `RouteBusy` | new |

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-runtime --all-targets --features tokio-runtime --locked routed_wait_does_not_hold_participant
./tools/cargo-dev test -p causlane-runtime --all-targets --features tokio-runtime --locked
./tools/cargo-dev fmt --all --check
./tools/cargo-dev clippy -p causlane-runtime --all-targets --features tokio-runtime --locked -- -D warnings
tools/pre-publication-review-gate
tools/formal-discipline-check --profile all --no-diff --json
git diff --check
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue: M2 partition-state retention semantics remain separate
  publication-track work.
