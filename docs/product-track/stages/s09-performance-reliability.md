# S09 — Performance, reliability и high-throughput readiness

**Status:** `active_next`

**Purpose:** Сохранить correctness, но не превратить hot path в тяжелый workflow/control-plane overhead.

## Milestones

### M09.1 — Bench suite

- **Status:** `done_or_near_done`
- **Outcome:** Criterion baseline for registry normalize, plan_hash, bundle load, replay verify, frontier conflict selection, lease grant, barrier append, explain.
- **Definition of done:**
  - `crates/causlane` exposes a Criterion bench target for the baseline surfaces;
  - `just bench-dispatch-baseline-build` compiles the harness without running measurements;
  - `just bench-dispatch-baseline` runs the local measurement suite;
  - `docs/product-track/bench-suite-matrix.json` records the benchmark IDs and threshold policy.

### M09.2 — Partitioned dispatcher

- **Status:** `done_or_near_done`
- **Outcome:** Host dispatch v2 partition routes and in-process admission coordinator with deterministic cross-partition ordering.
- **Definition of done:**
  - `HostTaskSpec` carries a typed `PartitionRoute`;
  - `PartitionRoute::acquisition_order()` is the single ordering helper;
  - `InProcessRuntime` coordinates admission for routed submits;
  - docs/spec/ADR/FIR describe admission-only semantics and non-goals.

### M09.3 — Batched durability

- **Status:** `done_or_near_done`
- **Outcome:** `AuditLogPort::append_batch` gives all-or-nothing ordered group commit over the existing audit boundary.
- **Definition of done:**
  - typed batch contract exists on `AuditLogPort`;
  - in-memory, SQLite, Postgres and tracing adapters use the same batch semantics;
  - write-ahead order and rollback negative controls are covered;
  - docs/ADR/FIR and adapter evidence match the implementation.

### M09.4 — Backpressure policy

- **Status:** `done_or_near_done`
- **Outcome:** Runtime-local wait/fail-fast overload policy over bounded in-process partition queues.
- **Definition of done:**
  - `InProcessBackpressurePolicy` exposes `Wait` and `FailFast` modes;
  - `submit`, `submit_routed`, `try_submit`, `try_submit_routed` and explicit policy APIs share one admission helper;
  - docs/ADR/FIR and runtime tests cover the overload surface;
  - no prose-only claim remains for protocol-critical behavior.

### M09.5 — Plan/template caches

- **Status:** `done_or_near_done`
- **Outcome:** Pure in-memory plan/template cache keyed by canonical plan material and compile snapshot refs.
- **Definition of done:**
  - `PlanTemplateCache` exposes a typed key, snapshot refs, hit/miss lookup and key hash;
  - cache entries reuse `PlanHashMaterial::compute_plan_hash` and `impact_set_hash`;
  - docs/ADR/FIR and contract tests cover cache reuse and stale-key negative controls;
  - no prose-only claim remains for protocol-critical behavior.

### M09.6 — Operational SLOs

- **Status:** `done_or_near_done`
- **Outcome:** Typed operational SLO measurement catalog for submit/admission/barrier/replay/explain p50/p95, queue depth and stale snapshot age.
- **Definition of done:**
  - `causlane-runtime` exposes `OPERATIONAL_SLO_METRICS`;
  - `validate_operational_slo_catalog` checks duplicate ids, missing metrics and shape drift;
  - docs/ADR/FIR describe host-defined thresholds and non-enforcement;
  - runtime tests cover stable tokens and negative controls.

### M09.7 — Chaos/recovery tests

- **Status:** `done_or_near_done`
- **Outcome:** Bounded in-process chaos/recovery evidence for slow handlers,
  host-owned retry, provider failure, route contention and ephemeral partition
  restart.
- **Definition of done:**
  - `causlane-runtime` recovery tests cover the bounded M09.7 scenarios;
  - `docs/product-track/chaos-recovery-matrix.json` records machine-readable
    evidence and residual risks;
  - ADR-0025 documents retry/recovery limits without claiming durable runtime
    authority;
  - no prose-only claim remains for the covered protocol-critical behavior.

## Exit gate

Есть benchmark profile, partitioned dispatcher, bounded queues, batched durable writes и typed SLO catalog для admission/barrier/replay/explain.
