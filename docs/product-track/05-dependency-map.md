# 05. Dependency map

## Critical path

```text
S00 Product Charter
  -> S01 Contract/Formal Readiness
  -> S02 Formal Models v1
  -> S03 Reference Kernel
  -> S04 Replay/Contract Testing
  -> S05 Constraint/Frontier Engine
  -> S06 AuthZ/Policy
  -> S07 Explainability/DX
  -> S08 Runtime Adapters
  -> S09 Performance/Reliability
  -> S10 Formal Depth
  -> S11 Alpha
  -> S12 Beta
  -> S13 1.0
```

## Non-linear workstreams

- Service/tooling cleanup runs continuously from S01 to S13.
- Formal exceptions policy must exist before formal coverage is public-facing.
- Replay oracle should precede runtime adapters.
- Shadow mode should precede production enforcement.
- Adapter certification should precede adapter docs that imply production use.
- Benchmarking should start before runtime implementation decisions harden.

## Hard blockers

| Work item | Blocks |
|---|---|
| Compiled bundle/formal IR | Formal artifacts, replay strict mode, codegen |
| Canonical hashing | replay, approval binding, stale-check, receipts |
| TruthAnchor/WitnessRef/LeaseRef | replay, formal checks, barrier/capability |
| Authz default deny | security model, RuntimeExecution barrier |
| mergeable() semantics | safe frontier, I-006 |
| formal receipts/stale-check | any claim of formal coverage |
| lifecycle reducer in replay/runtime | kernel/runtime consistency |
| adapter certification | runtime ecosystem |

## Parallelizable tracks

Can run in parallel after S01:

- Alloy model hardening;
- P protocol model hardening;
- CLI/explain UX design;
- docs/cookbook writing;
- benchmark harness design;
- Authz adapter spike;
- observability connector spike.

Do not run before S01:

- production adapters;
- public API stabilization;
- distributed scheduler/service mode;
- marketing claims about formal guarantees.
