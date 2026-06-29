# S12 — Beta integrations и ecosystem validation

**Status:** `active_next`

**Purpose:** Проверить полезность на нескольких реальных интеграциях и улучшить DX/adapters.

## Milestones

### M12.1 — Reference integration 1

- **Status:** `done_or_near_done`
- **Outcome:** Rust service with API+worker+audit+projection.
- **Evidence:** `examples/reference-integration` runs an in-repo API+worker+audit+projection slice through public host-dispatch, runtime audit and guarded projection APIs; `docs/product-track/reference-integration-matrix.json` is checked by `tools/reference-integration-check`.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.2 — Reference integration 2

- **Status:** `done_or_near_done`
- **Outcome:** Agent/tool execution or CI/CD/release orchestration.
- **Evidence:** `examples/release-orchestration` runs a bounded CI/CD release orchestration graph through public host-dispatch and runtime audit APIs; `docs/product-track/reference-integration-matrix.json` is checked by `tools/reference-integration-check`.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.

### M12.3 — Migration/shadow docs

- **Status:** `done_or_near_done`
- **Outcome:** How to adopt incrementally without rewrite.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.
- **Evidence:** `docs/scenarios/migration-shadow.md` and
  `docs/product-track/migration-shadow-adoption.json` document the bounded
  migration/shadow adoption path; `tools/migration-shadow-doc-check` pins the
  doc to existing shadow API and runtime negative-control tests.

### M12.4 — Adapter ecosystem

- **Status:** `done_or_near_done`
- **Outcome:** Document external adapter interface, compatibility/certification.
- **Definition of done:**
  - typed contract or executable check exists;
  - docs/ADR updated;
  - tests or replay scenario added where relevant;
  - no prose-only claim remains for protocol-critical behavior.
- **Evidence:** `docs/scenarios/adapter-ecosystem.md` and
  `docs/product-track/adapter-ecosystem-guide.json` document the external
  adapter interface and certification expectations; `tools/adapter-ecosystem-doc-check`
  pins those claims to the existing M08.7 certification matrix.

### M12.5 — API validation loop

- **Status:** `planned`
- **Outcome:** Closed loop over realistic synthetic examples, property/fuzz testing and performance scale testing before API freeze.
- **Evidence seed:** `docs/product-track/api-validation-loop-plan.json`
  records the selected candidate surfaces and binds them to terminal
  `accepted_for_freeze` classifications for every selected surface.
  `examples/facade-kernel-ergonomics` is the first facade-only synthetic example
  for `public_facade_and_core_kernel`; `examples/facade-kernel-operator-workflow`
  adds a near-real facade-only operator workflow for the same surface.
  `facade_kernel_frontier` seeds the same surface's property/fuzz lane. A
  15-minute dispatcher long-run for that fuzz target is recorded in
  `docs/formal/impact/2026-06-29-m12-5-facade-fuzz-long-run.md`; dispatcher
  Criterion evidence is recorded in
  `docs/formal/impact/2026-06-29-m12-5-facade-performance-scale.md`; and the
  facade/kernel surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-facade-api-feedback-classification.md`.
  `examples/replay-diagnostics` adds replay/explain diagnostics coverage for
  `replay_scenario_explain`; `examples/replay-operator-diagnostics` adds a
  near-real replay diagnostics workflow for the same surface. A 15-minute
  dispatcher long-run for `replay_trace_json` and `replay_scenario_yaml` is
  recorded in `docs/formal/impact/2026-06-29-m12-5-replay-fuzz-long-run.md`,
  dispatcher Criterion evidence is recorded in
  `docs/formal/impact/2026-06-29-m12-5-replay-performance-scale.md`, and the
  replay surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-replay-api-feedback-classification.md`.
  `examples/contracts-boundary-ergonomics` adds
  contracts/bundle/plan-hash coverage for `contracts_registry_bundle_plan_hash`;
  `examples/contracts-registry-bundle-workflow` adds a near-real multi-predicate
  contracts workflow for the same surface. A 15-minute dispatcher long-run for
  `registry_yaml_compile` is recorded in
  `docs/formal/impact/2026-06-29-m12-5-contracts-fuzz-long-run.md`, dispatcher
  Criterion evidence is recorded in
  `docs/formal/impact/2026-06-29-m12-5-contracts-performance-scale.md`, and the
  contracts surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-contracts-api-feedback-classification.md`.
  `examples/runtime-guarded-audit-projection` adds guarded execution, audit trace
  projection and projection-redaction coverage for
  `runtime_dispatch_audit_projection`; `examples/runtime-operator-workflow`
  adds a multi-operation runtime host workflow for the same surface.
  `runtime_guarded_audit_projection` seeds the same surface's property/fuzz
  lane. A 15-minute dispatcher long-run for that fuzz target is recorded in
  `docs/formal/impact/2026-06-29-m12-5-runtime-fuzz-long-run.md`; the surface
  also has dispatcher performance-scale evidence recorded in
  `docs/formal/impact/2026-06-29-m12-5-runtime-performance-scale.md`; the
  runtime surface is now classified `accepted_for_freeze` in
  `docs/formal/impact/2026-06-29-m12-5-runtime-api-feedback-classification.md`.
- **Definition of done:**
  - realistic synthetic corpus exists and covers common public API workflows;
  - property/fuzz lanes exist for the surfaces selected for freeze;
  - scale/performance evidence exists for the hot paths selected for freeze;
  - API feedback from all three lanes is classified as `needs_api_change` or `accepted_for_freeze`.

### M12.6 — Semver pre-1.0 freeze plan

- **Status:** `planned`
- **Outcome:** Identify APIs slated for stabilization after M12.5 validation evidence is classified.
- **Definition of done:**
  - M12.5 API validation loop has a recorded terminal classification;
  - APIs accepted for freeze are listed explicitly;
  - APIs needing change remain experimental or get a pre-freeze action;
  - no freeze claim is made from prose-only evidence.

## Exit gate

Есть 2–3 reference integrations, migration/shadow-mode story и feedback-driven API hardening.
