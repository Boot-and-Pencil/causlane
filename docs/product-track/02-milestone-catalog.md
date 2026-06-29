# 02. Milestone catalog

## M00.1 — Product charter

- **Stage:** S00
- **Status:** `done_or_near_done`
- **Purpose:** Зафиксировать positioning, target users, non-goals, first use cases, success metrics.

## M00.2 — Glossary freeze v0.1

- **Stage:** S00
- **Status:** `done_or_near_done`
- **Purpose:** Уточнить ActionCall/ActionPlan/Op/Impact/Witness/Anchor/Lease/Barrier/Projection/Overlay/Constraint.

## M00.3 — ADR baseline

- **Stage:** S00
- **Status:** `done_or_near_done`
- **Purpose:** Поддерживать ADR chain: docs-first, SSOT, hexagonal architecture, authz default deny, plan hash, anchor/witness, merge protocol.

## M00.4 — Context hygiene

- **Stage:** S00
- **Status:** `ongoing`
- **Purpose:** Контекст-паки, generated files, formal artifacts и секреты должны иметь hygiene/check tools.

## M01.1 — Toolchain doctor

- **Stage:** S01
- **Status:** `exists_harden`
- **Purpose:** Проверка Rust, Cargo, Java, Alloy, P, Kani, Verus, Lean4, Z3, jq, python3, just; machine-readable report.

## M01.2 — Formal install/provisioning

- **Stage:** S01
- **Status:** `exists_harden`
- **Purpose:** Reproducible установка/проверка инструментов, pin versions, SHA/checksum, offline/CI story.

## M01.3 — Canonical serialization v1

- **Stage:** S01
- **Status:** `exists`
- **Purpose:** Canonical JSON, content_hash, bundle_hash, plan_hash, impact_set_hash, stable formatting.

## M01.4 — Bundle formal input v0.2

- **Stage:** S01
- **Status:** `exists_harden`
- **Purpose:** CompiledDispatchBundle содержит predicates, profiles, routes, effects, claims, witness requirements, authz policies, formal obligations.

## M01.5 — Scenario catalog v1

- **Stage:** S01
- **Status:** `exists_expand`
- **Purpose:** Executable *.scenario.yaml + generated traces + positive/negative controls.

## M01.6 — Replay oracle strict bundle mode

- **Stage:** S01
- **Status:** `exists_expand`
- **Purpose:** Replay проверяет trace against bundle, plan_hash, witnesses, anchors, leases, lifecycle, authz, claims.

## M01.7 — Receipts/stale-check v2

- **Stage:** S01
- **Status:** `exists_harden`
- **Purpose:** Codegen/tool-run receipts, generated artifact hashes, stale-check-all, coverage derivation.

## M02.1 — Alloy structural checks v1

- **Stage:** S02
- **Status:** `exists_harden`
- **Purpose:** Generated facts + generic core assertions for lifecycle, anchors, route/profile, conflict frontier, witness/approval binding.

## M02.2 — P protocol monitors v1

- **Stage:** S02
- **Status:** `exists_harden`
- **Purpose:** Generated monitors for barrier-before-execution, drain/admission races, retry/idempotency, lease release/expiry.

## M02.3 — Kani bounded harnesses v1

- **Stage:** S02
- **Status:** `exists_harden`
- **Purpose:** Generated/handwritten harnesses for reducers, trace validation, lease table, capability derivation, parser boundaries.

## M02.4 — Verus proof facet v1

- **Stage:** S02
- **Status:** `exists_harden`
- **Purpose:** Abstract preservation proofs: lifecycle, overlay monotonicity, replay soundness, lease map invariants.

## M02.5 — Lean4 proof applications v1

- **Stage:** S02
- **Status:** `exists_harden`
- **Purpose:** Generated scenario-bound theorem applications checked in the full formal gate.

## M02.6 — Negative controls discipline

- **Stage:** S02
- **Status:** `exists_harden`
- **Purpose:** Each invariant lane has expected-failure controls; accidental failure is not evidence.

## M02.7 — Formal exceptions policy

- **Stage:** S02
- **Status:** `exists_harden`
- **Purpose:** Every allowed formal-lane exception has owner, rationale, expiry, profile, allowed scope.

## M03.1 — Pure lifecycle reducer

- **Stage:** S03
- **Status:** `done_or_near_done`
- **Purpose:** No async/I/O; transition(state,event,contracts)->decision; used by replay/runtime/tests.

## M03.2 — Consequence profile obligations

- **Stage:** S03
- **Status:** `done_or_near_done`
- **Purpose:** RuntimeExecution/ProjectionRead/Oversight/Topology/Evidence obligations in code and bundle.

## M03.3 — Effect signature core

- **Stage:** S03
- **Status:** `done_or_near_done`
- **Purpose:** reads/writes/produces/requires/invalidates/conflict_domains/claims.

## M03.4 — Barrier/capability core

- **Stage:** S03
- **Status:** `done_or_near_done`
- **Purpose:** ExecutionBarrier, ExecutionCapability, scoped execution permission, executor guard.

## M03.5 — LeaseTable core

- **Stage:** S03
- **Status:** `done_or_near_done`
- **Purpose:** Exclusive/shared/token leases, no-overlap, expiry/revocation, claim coverage.

## M03.6 — Domain errors and stable codes

- **Stage:** S03
- **Status:** `done_or_near_done`
- **Purpose:** Typed errors with stable codes for replay/explain/CLI/tests.

## M04.1 — Golden success scenarios

- **Stage:** S04
- **Status:** `done_or_near_done`
- **Purpose:** release_promote, approval, projection, conflict-free parallelism, read-only sidecar.

## M04.2 — Negative scenario suite

- **Stage:** S04
- **Status:** `planned`
- **Purpose:** execution_without_barrier, observed_without_execution, projection_without_anchor, wrong plan, missing witness, conflicting leases.

## M04.3 — Scenario-to-trace compiler

- **Stage:** S04
- **Status:** `done_or_near_done`
- **Purpose:** One scenario source emits trace, formal facts, replay expectation.

## M04.4 — Replay explain output

- **Stage:** S04
- **Status:** `done_or_near_done`
- **Purpose:** Replay failures return exact invariant/error and causal location.

## M04.5 — Contract testing harness

- **Stage:** S04
- **Status:** `done_or_near_done`
- **Purpose:** dispatch_test! / YAML runner for predicates and adapters.

## M04.6 — Mutation/fuzz tests

- **Stage:** S04
- **Status:** `done_or_near_done`
- **Purpose:** Generate malformed traces and small worlds; ensure fail-closed.

## M05.1 — Tier model

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** Admission/planning/dispatch/barrier/execution/observation/projection/closure as authority stages.

## M05.2 — Lane model

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** Lane capacity/capability/fairness without semantic authority.

## M05.3 — ConstraintSpec/Provider

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** Requirement/Claim/Lease, snapshots/epochs, Allow/Wait/Deny/Restrict decisions.

## M05.4 — Graph indexes

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** wait_by_fact, wait_by_scope, active_by_write_scope, ready_by_lane, incremental rebuild.

## M05.5 — Safe frontier selection

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** Ready antichain, no hard deps, no conflicts, lane/resource budgets.

## M05.6 — Drain/fence protocol

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** domain/global drain epochs, safe points, disjoint domains, frozen sidecars.

## M05.7 — Runtime constraint updates

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** capacity/quota/freeze/rate-limit updates with epoch, rebuild frontier, no truth rewrite.

## M05.8 — why-not-parallel

- **Stage:** S05
- **Status:** `done_or_near_done`
- **Purpose:** Machine-readable blocker/rationale for concurrency decisions.

## M06.1 — Authz policy model

- **Stage:** S06
- **Status:** `done_or_near_done`
- **Purpose:** AuthzPolicy, stages, policy_id/version, freshness/expiry, default deny.

## M06.2 — Cedar adapter prototype

- **Stage:** S06
- **Status:** `done_or_near_done`
- **Purpose:** Embedded PDP mapping Predicate/Subject/Context to Cedar Principal/Action/Resource/Context.

## M06.3 — Casbin/AuthZEN/OpenFGA sketches

- **Stage:** S06
- **Status:** `done_or_near_done`
- **Purpose:** Adapter contracts; do not bake one engine into core.

## M06.4 — Approval as action

- **Stage:** S06
- **Status:** `done_or_near_done`
- **Purpose:** gate.approve/gate.deny bound to action_id+plan_hash+impact_set_hash.

## M06.5 — Step-up and SoD

- **Stage:** S06
- **Status:** `done_or_near_done`
- **Purpose:** MFA/step-up, separation-of-duties, approval freshness.

## M06.6 — Execution capability enforcement

- **Stage:** S06
- **Status:** `done_or_near_done`
- **Purpose:** Worker executes only with scoped capability derived from barrier.

## M06.7 — Projection access/redaction

- **Stage:** S06
- **Status:** `done_or_near_done`
- **Purpose:** Read/projection authz, sensitive-field redaction policy.

## M07.1 — CLI explain/why

- **Stage:** S07
- **Status:** `done_or_near_done`
- **Purpose:** explain, why-blocked, why-not-parallel, graph export, replay diagnostics.

## M07.2 — Graph export

- **Stage:** S07
- **Status:** `done_or_near_done`
- **Purpose:** Mermaid/DOT/JSON graph slices with blockers/witnesses/leases.

## M07.3 — Tracing connector

- **Stage:** S07
- **Status:** `done_or_near_done`
- **Purpose:** Structured action/op spans; logs derived, not truth.

## M07.4 — OpenTelemetry optional

- **Stage:** S07
- **Status:** `done_or_near_done`
- **Purpose:** OTLP logs/traces/metrics adapter; fail-open for telemetry only.

## M07.5 — Redaction policy

- **Stage:** S07
- **Status:** `done_or_near_done`
- **Purpose:** Audit/log/projection/replay/support-bundle redaction classes.

## M07.6 — Support bundle

- **Stage:** S07
- **Status:** `done_or_near_done`
- **Purpose:** Sanitized bundle with trace, graph slice, route rationale, environment/tool report.

## M07.7 — Cookbook docs

- **Stage:** S07
- **Status:** `done_or_near_done`
- **Purpose:** Add action, approval, conflict, drain, replay, adapter, authz, projection recipes.

## M08.1 — In-process runtime

- **Stage:** S08
- **Status:** `done_or_near_done`
- **Purpose:** Tokio partition loops, bounded queues, semaphores for capacity, no global lock.

## M08.2 — Audit adapters

- **Stage:** S08
- **Status:** `done_or_near_done`
- **Purpose:** In-memory, SQLite, Postgres append-only audit; group commit policies.

## M08.3 — Executor port/adapters

- **Stage:** S08
- **Status:** `done_or_near_done`
- **Purpose:** Tower-like Service port; hard effects only with capability.

## M08.4 — Apalis adapter

- **Stage:** S08
- **Status:** `done_or_near_done`
- **Purpose:** Rust-native jobs backend; certification tests.

## M08.5 — Restate adapter

- **Stage:** S08
- **Status:** `done_or_near_done`
- **Purpose:** Durable handler/workflow adapter; optional feature.

## M08.6 — Temporal/Dapr/Conductor adapters

- **Stage:** S08
- **Status:** `future`
- **Purpose:** Experimental/community boundary; no hard dependency.

## M08.7 — Adapter certification

- **Stage:** S08
- **Status:** `done_or_near_done`
- **Purpose:** Bounded certification matrix for existing adapters; retry/cancel/truth orchestration deferred.

## M08.8 — Shadow mode

- **Stage:** S08
- **Status:** `done_or_near_done`
- **Purpose:** Runtime shadow comparer over `InProcessRuntimeEvent`; compare expected vs actual without enforcement.

## M09.1 — Bench suite

- **Stage:** S09
- **Status:** `done_or_near_done`
- **Purpose:** Criterion baseline for registry normalize, plan_hash, bundle load, replay verify, frontier conflict selection, lease grant, barrier append, explain.

## M09.2 — Partitioned dispatcher

- **Stage:** S09
- **Status:** `done_or_near_done`
- **Purpose:** Host dispatch v2 partition routes and in-process admission coordinator with deterministic cross-partition ordering.

## M09.3 — Batched durability

- **Stage:** S09
- **Status:** `done_or_near_done`
- **Purpose:** `AuditLogPort::append_batch` gives all-or-nothing ordered group commit over the existing audit boundary.

## M09.4 — Backpressure policy

- **Stage:** S09
- **Status:** `done_or_near_done`
- **Purpose:** Runtime-local wait/fail-fast overload policy over bounded in-process partition queues.

## M09.5 — Plan/template caches

- **Stage:** S09
- **Status:** `done_or_near_done`
- **Purpose:** Pure in-memory plan/template cache keyed by canonical plan material and compile snapshot refs.

## M09.6 — Operational SLOs

- **Stage:** S09
- **Status:** `done_or_near_done`
- **Purpose:** Typed operational SLO measurement catalog for submit/admission/barrier/replay/explain p50/p95, queue depth and stale snapshot age.

## M09.7 — Chaos/recovery tests

- **Stage:** S09
- **Status:** `done_or_near_done`
- **Purpose:** Bounded in-process chaos/recovery evidence for slow handlers, host-owned retry, provider failure, route contention and ephemeral partition restart.

## M10.1 — Invariant expansion

- **Stage:** S10
- **Status:** `exists`
- **Purpose:** Shared active/planned/known invariant-id catalog and planned I-011..I-020 reservations without coverage credit.

## M10.2 — P interleavings depth

- **Stage:** S10
- **Status:** `exists_expand`
- **Purpose:** P-first bounded controls for retry duplicate execution, authz revocation and constraint epoch; cancellation/partition remain planned.

## M10.3 — Kani integration

- **Stage:** S10
- **Status:** `exists_expand`
- **Purpose:** cargo kani profile, unwind bounds, generated fixtures, CI/nightly split.

## M10.4 — Verus/Lean proof hardening

- **Stage:** S10
- **Status:** `exists_expand`
- **Purpose:** Verus and Lean4 proof lanes have a machine-readable always-blocking contract; deeper proof semantics remain expansion work.

## M10.5 — Coverage anti-theatre

- **Stage:** S10
- **Status:** `exists_expand`
- **Purpose:** Coverage matrix Markdown is fully drift-checked from the receipt-derived report; active docs point to generated coverage instead of hand-maintained live inventories.

## M10.6 — Proof/refinement docs

- **Stage:** S10
- **Status:** `exists_expand`
- **Purpose:** Schema-validated proof/refinement scope classifies claim strength, and generated Markdown drift is checked by the formal gate.

## M11.1 — Crate naming/publish check

- **Stage:** S11
- **Status:** `exists_expand`
- **Purpose:** No-publish readiness cleared deterministic local blockers and handed off to staged release evidence.

## M11.2 — Feature flags

- **Stage:** S11
- **Status:** `done_or_near_done`
- **Purpose:** default minimal; existing optional runtime integrations are explicit and non-default.

## M11.3 — Public API review

- **Stage:** S11
- **Status:** `done_or_near_done`
- **Purpose:** Rust API guidelines, builders, newtypes, no raw Strings on critical fields.

## M11.4 — Examples

- **Stage:** S11
- **Status:** `done_or_near_done`
- **Purpose:** simple-local, approval-gate, consequence-parallelism and why-not-parallel are runnable and checked.

## M11.5 — Security/release hygiene

- **Stage:** S11
- **Status:** `done_or_near_done`
- **Purpose:** licenses, dependency audit, context-pack scan, secret rules, vulnerability policy.

## M11.6 — Contributor guide

- **Stage:** S11
- **Status:** `done_or_near_done`
- **Purpose:** public contributor guide consolidates ADR process, new predicate checklist, formal obligation template, adapter certification and AI accountability.

## M11.7 — Release notes

- **Stage:** S11
- **Status:** `done_or_near_done`
- **Purpose:** Clear limitations: not workflow engine, formal lanes coverage, unstable APIs.

## M12.1 — Reference integration 1

- **Stage:** S12
- **Status:** `done_or_near_done`
- **Purpose:** Rust service with API+worker+audit+projection.

## M12.2 — Reference integration 2

- **Stage:** S12
- **Status:** `done_or_near_done`
- **Purpose:** Agent/tool execution or CI/CD/release orchestration.

## M12.3 — Migration/shadow docs

- **Stage:** S12
- **Status:** `done_or_near_done`
- **Purpose:** How to adopt incrementally without rewrite.

## M12.4 — Adapter ecosystem

- **Stage:** S12
- **Status:** `done_or_near_done`
- **Purpose:** Document external adapter interface, compatibility/certification.

## M12.5 — API validation loop

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** Closed loop over realistic synthetic examples, property/fuzz testing and performance scale testing before API freeze.
- **Evidence seed:** selected-surface inventory is recorded in
  `docs/product-track/api-validation-loop-plan.json`; every selected surface now
  has terminal `accepted_for_freeze` classification evidence.
  `examples/facade-kernel-ergonomics` seeds facade-only synthetic evidence for
  `public_facade_and_core_kernel`; `examples/facade-kernel-operator-workflow`
  adds a near-real facade-only operator workflow for the same surface.
  `facade_kernel_frontier` seeds the same surface's property/fuzz lane. A
  15-minute dispatcher long-run for that fuzz target is recorded in
  `docs/formal/impact/2026-06-29-m12-5-facade-fuzz-long-run.md`; dispatcher
  Criterion evidence is recorded in
  `docs/formal/impact/2026-06-29-m12-5-facade-performance-scale.md`; and the
  facade/kernel surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-facade-api-feedback-classification.md`.
  `examples/replay-diagnostics` seeds replay/explain diagnostics evidence for
  `replay_scenario_explain`; `examples/replay-operator-diagnostics` adds a
  near-real replay diagnostics workflow for the same surface. A 15-minute
  dispatcher long-run for `replay_trace_json` and `replay_scenario_yaml` is
  recorded in `docs/formal/impact/2026-06-29-m12-5-replay-fuzz-long-run.md`,
  dispatcher Criterion evidence is recorded in
  `docs/formal/impact/2026-06-29-m12-5-replay-performance-scale.md`, and the
  replay surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-replay-api-feedback-classification.md`.
  `examples/contracts-boundary-ergonomics` seeds
  contracts/bundle/plan-hash evidence for `contracts_registry_bundle_plan_hash`;
  `examples/contracts-registry-bundle-workflow` adds a near-real multi-predicate
  contracts workflow for the same surface. A 15-minute dispatcher long-run for
  `registry_yaml_compile` is recorded in
  `docs/formal/impact/2026-06-29-m12-5-contracts-fuzz-long-run.md`, dispatcher
  Criterion evidence is recorded in
  `docs/formal/impact/2026-06-29-m12-5-contracts-performance-scale.md`, and the
  contracts surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-contracts-api-feedback-classification.md`.
  `examples/runtime-guarded-audit-projection` seeds guarded execution, audit trace
  projection and projection-redaction evidence for
  `runtime_dispatch_audit_projection`; `examples/runtime-operator-workflow`
  adds a multi-operation runtime host workflow for the same surface.
  `runtime_guarded_audit_projection` seeds the same surface's property/fuzz
  lane. A 15-minute dispatcher long-run for that fuzz target is recorded in
  `docs/formal/impact/2026-06-29-m12-5-runtime-fuzz-long-run.md`; the surface
  also has dispatcher performance-scale evidence recorded in
  `docs/formal/impact/2026-06-29-m12-5-runtime-performance-scale.md`; the
  runtime surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-runtime-api-feedback-classification.md`.

## M12.6 — Semver pre-1.0 freeze plan

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** Identify APIs slated for stabilization after M12.5 validation evidence is classified.

## M13.1 — API stabilization

- **Stage:** S13
- **Status:** `planned`
- **Purpose:** Core types, bundle format, event model, replay trace, adapter traits.

## M13.2 — Compatibility policy

- **Stage:** S13
- **Status:** `planned`
- **Purpose:** Schema migration, trace replay across versions, deprecation windows.

## M13.3 — Formal/replay release gate

- **Stage:** S13
- **Status:** `planned`
- **Purpose:** No release unless formal-ready, replay corpus, coverage matrix, exceptions valid.

## M13.4 — Production readiness docs

- **Stage:** S13
- **Status:** `planned`
- **Purpose:** Operations, performance tuning, failure modes, backup/restore audit.

## M13.5 — Security model docs

- **Stage:** S13
- **Status:** `planned`
- **Purpose:** Threat model, authz defaults, capability scope, redaction, supply chain.

## M13.6 — 1.0 release

- **Stage:** S13
- **Status:** `planned`
- **Purpose:** Publish stable crates and docs with honest guarantees.

## M14.1 — Service/control-plane mode

- **Stage:** S14
- **Status:** `future`
- **Purpose:** Optional server mode; still small kernel authority.

## M14.2 — Dashboard/UI

- **Stage:** S14
- **Status:** `future`
- **Purpose:** Visual graph, blockers, audit/replay exploration.

## M14.3 — Advanced distributed coordination

- **Stage:** S14
- **Status:** `future`
- **Purpose:** Multi-node scheduler, HA audit, failover only after formal model/design.

## M14.4 — Adapter marketplace/catalog

- **Stage:** S14
- **Status:** `future`
- **Purpose:** Community adapters with certification receipts.

## M14.5 — Advanced proof/language support

- **Stage:** S14
- **Status:** `future`
- **Purpose:** More proof generators/languages as generated projections, not manual authority.
