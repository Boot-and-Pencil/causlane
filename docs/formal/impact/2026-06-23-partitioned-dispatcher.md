# Formal Impact Record: M09.2 partitioned dispatcher

## Change metadata

- Change ID: FIR-2026-06-23-partitioned-dispatcher
- PR/issue: M09.2 Partitioned dispatcher
- Owner: repo maintainers
- Date: 2026-06-23
- Impact class: F2 (runtime admission coordination)

## Touched protocol-critical paths

```text
crates/causlane-core/src/integration.rs
crates/causlane-runtime/src/in_process/mod.rs
crates/causlane-runtime/src/in_process/coordinator.rs
crates/causlane-runtime/src/in_process/tests.rs
docs/specs/host-dispatch-api-v2.md
docs/adr/0017-host-dispatch-api-v2-partition-coordinator.md
```

## Summary

This change introduces host dispatch API v2 with typed partition routing and an
in-process admission coordinator. The coordinator acquires route permits in the
single deterministic order produced by `PartitionRoute::acquisition_order()`,
admits the task to its primary partition, and releases permits before host
effect execution.

## Affected invariants

```text
I-001: unchanged - execution still happens only through host handlers.
I-006: unchanged - lease conflict semantics are not modified.
I-007: unchanged - drain fence semantics are not modified.
I-008: unchanged - lifecycle/replay semantics are not modified.
I-009: unchanged - plan/impact/witness/authz binding semantics are unchanged.
new invariant ids: none
```

## Affected formal models

```text
none - no Formal IR schema, generated model artifact, scenario, or replay trace
schema changes.
```

## Affected protocols

```text
PR-host-dispatch-v2: host-facing dispatch tasks now carry a PartitionRoute.
InProcessRuntime coordinates admission only; durable distributed scheduling and
lease transactions remain out of scope.
```

## Contract changes

- Host API version changed from `causlane.host-dispatch.v1` to `causlane.host-dispatch.v2`.
- `HostTaskSpec` added a required `partition_route`.
- `HostDispatcherCapabilities` added `supports_partition_coordination`.
- `HostDispatchError` added `InvalidPartitionRoute`.
- Runtime added routed submit helpers and `RouteBusy` admission diagnostics.
- Bundle, Formal IR, replay trace/scenario and receipt schemas: unchanged.

## Required negative controls

| Scenario | Expected lane | Expected error/monitor/check | Status |
|---|---|---|---|
| v1 task submitted to v2 validator | core test | `UnsupportedApiVersion` | new |
| empty route primary/participant | core test | `InvalidPartitionRoute` | new |
| explicit partition differs from route primary | runtime test | `InvalidPartitionRoute` and rejected event | new |
| unknown participant partition | runtime test | `UnknownPartition` before admission | new |
| route permit already held | coordinator test | `RouteBusy` and event | new |
| reversed multi-partition routes | runtime test | both routed admissions complete | new |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Rust | host dispatch v2 and in-process coordinator tests | no | rust |
| Product docs | host-dispatch-api-v2 + ADR-0017 | no | docs |
| Alloy | unchanged | yes | rust |
| P | unchanged | yes | rust |
| Kani | unchanged | yes | proof/all |
| Verus | unchanged | yes | proof/all |
| Lean4 | unchanged | yes | proof/all |

## Not applicable lanes

Formal models are not regenerated because this change does not modify replay,
bundle, scenario, Formal IR or invariant semantics. The executable evidence is
the Rust admission-coordinator test suite.

## Acceptance commands

```bash
./tools/cargo-dev fmt --all --check
just bench-m09-1-build
just check
just clippy
just test
just coverage-clean && just coverage
./tools/formal-discipline-check --profile base --changed-files /tmp/causlane-m09-2-partitioned-dispatcher-changed-files.txt
./tools/formal-exceptions-check
./tools/schema-validate-all
./tools/product-track-status-check
just formal-coverage-matrix-check
just bench-m09-1
```

## Exception request

- Exception needed? no
- Exception id:
- Allowed profiles:
- Forbidden profiles:
- Expiry date:
- Follow-up issue:
