# 01. Product track map

## S00 — Контекст, назначение и продуктовая рамка

**Status:** `baseline_exists`

**Theme:** Зафиксировать смысл продукта, non-goals, целевую аудиторию, язык и границы.

**Milestones:**

- `M00.1` — **Product charter** (`done_or_near_done`): Зафиксировать positioning, target users, non-goals, first use cases, success metrics.
- `M00.2` — **Glossary freeze v0.1** (`done_or_near_done`): Уточнить ActionCall/ActionPlan/Op/Impact/Witness/Anchor/Lease/Barrier/Projection/Overlay/Constraint.
- `M00.3` — **ADR baseline** (`done_or_near_done`): Поддерживать ADR chain: docs-first, SSOT, hexagonal architecture, authz default deny, plan hash, anchor/witness, merge protocol.
- `M00.4` — **Context hygiene** (`ongoing`): Контекст-паки, generated files, formal artifacts и секреты должны иметь hygiene/check tools.

**Exit gate:** Команда может одним документом объяснить, что такое Causlane, чем он не является, и почему он не конкурирует с workflow/job engines.

## S01 — Контрактный фундамент и formal readiness

**Status:** `advanced_in_repo`

**Theme:** Стабилизировать compiled bundle, canonical hash, scenario/replay, formal IR, receipts и stale-check.

**Milestones:**

- `M01.1` — **Toolchain doctor** (`exists_harden`): Проверка Rust, Cargo, Java, Alloy, P, Kani, Verus, Lean4, Z3, jq, python3, just; machine-readable report.
- `M01.2` — **Formal install/provisioning** (`exists_harden`): Reproducible установка/проверка инструментов, pin versions, SHA/checksum, offline/CI story.
- `M01.3` — **Canonical serialization v1** (`exists`): Canonical JSON, content_hash, bundle_hash, plan_hash, impact_set_hash, stable formatting.
- `M01.4` — **Bundle formal input v0.2** (`exists_harden`): CompiledDispatchBundle содержит predicates, profiles, routes, effects, claims, witness requirements, authz policies, formal obligations.
- `M01.5` — **Scenario catalog v1** (`exists_expand`): Executable *.scenario.yaml + generated traces + positive/negative controls.
- `M01.6` — **Replay oracle strict bundle mode** (`exists_expand`): Replay проверяет trace against bundle, plan_hash, witnesses, anchors, leases, lifecycle, authz, claims.
- `M01.7` — **Receipts/stale-check v2** (`exists_harden`): Codegen/tool-run receipts, generated artifact hashes, stale-check-all, coverage derivation.

**Exit gate:** `just formal-ready` и `just formal-verify-all` проходят в чистой среде; coverage matrix выводится из receipts, а не из prose.

## S02 — Формальные модели v1

**Status:** `advanced_in_repo`

**Theme:** Сделать Alloy/P/Kani/Verus/Lean4 не демонстрацией, а рабочим verification contour для ключевых инвариантов.

**Milestones:**

- `M02.1` — **Alloy structural checks v1** (`exists_harden`): Generated facts + generic core assertions for lifecycle, anchors, route/profile, conflict frontier, witness/approval binding.
- `M02.2` — **P protocol monitors v1** (`exists_harden`): Generated monitors for barrier-before-execution, drain/admission races, retry/idempotency, lease release/expiry.
- `M02.3` — **Kani bounded harnesses v1** (`exists_harden`): Generated/handwritten harnesses for reducers, trace validation, lease table, capability derivation, parser boundaries.
- `M02.4` — **Verus proof facet v1** (`exists_harden`): Abstract preservation proofs: lifecycle, overlay monotonicity, replay soundness, lease map invariants.
- `M02.5` — **Lean4 proof applications v1** (`exists_harden`): Generated scenario-bound theorem applications checked in the full formal gate.
- `M02.6` — **Negative controls discipline** (`exists_harden`): Each invariant lane has expected-failure controls; accidental failure is not evidence.
- `M02.7` — **Formal exceptions policy** (`exists_harden`): Every allowed formal-lane exception has owner, rationale, expiry, profile, allowed scope.

**Exit gate:** Для I-001/I-002/I-003/I-006/I-008/I-009 есть generated artifact + tool receipt + stale-check + negative controls.

## S03 — Reference kernel и executable semantics

**Status:** `advanced_in_repo`

**Theme:** Превратить контракт в маленькое чистое ядро: lifecycle reducer, guards, frontier abstractions, replay oracle.

**Milestones:**

- `M03.1` — **Pure lifecycle reducer** (`done_or_near_done`): No async/I/O; transition(state,event,contracts)->decision; used by replay/runtime/tests.
- `M03.2` — **Consequence profile obligations** (`done_or_near_done`): RuntimeExecution/ProjectionRead/Oversight/Topology/Evidence obligations in code and bundle.
- `M03.3` — **Effect signature core** (`done_or_near_done`): reads/writes/produces/requires/invalidates/conflict_domains/claims.
- `M03.4` — **Barrier/capability core** (`done_or_near_done`): ExecutionBarrier, ExecutionCapability, scoped execution permission, executor guard.
- `M03.5` — **LeaseTable core** (`done_or_near_done`): Exclusive/shared/token leases, no-overlap, expiry/revocation, claim coverage.
- `M03.6` — **Domain errors and stable codes** (`done_or_near_done`): Typed errors with stable codes for replay/explain/CLI/tests.

**Exit gate:** Core reducer и validators можно использовать без runtime/adapters; все protocol-critical функции покрыты unit/property/Kani checks.

## S04 — Сценарии, replay и contract testing

**Status:** `advanced_in_repo`

**Theme:** Расширить executable scenario catalog и replay oracle до уровня основного devtool.

**Milestones:**

- `M04.1` — **Golden success scenarios** (`done_or_near_done`): release_promote, approval, projection, conflict-free parallelism, read-only sidecar.
- `M04.2` — **Negative scenario suite** (`planned`): execution_without_barrier, observed_without_execution, projection_without_anchor, wrong plan, missing witness, conflicting leases.
- `M04.3` — **Scenario-to-trace compiler** (`done_or_near_done`): One scenario source emits trace, formal facts, replay expectation.
- `M04.4` — **Replay explain output** (`done_or_near_done`): Replay failures return exact invariant/error and causal location.
- `M04.5` — **Contract testing harness** (`done_or_near_done`): dispatch_test! / YAML runner for predicates and adapters.
- `M04.6` — **Mutation/fuzz tests** (`done_or_near_done`): Generate malformed traces and small worlds; ensure fail-closed.

**Exit gate:** Каждое новое predicate/feature требует scenario + replay expectation + formal obligation/exemption.

## S05 — Constraint Plane, graph/frontier и lanes/tiers

**Status:** `advanced_in_repo`

**Theme:** Реализовать безопасную автоматическую диспетчеризацию параллельности на основе consequences, claims, leases, witnesses и constraints.

**Milestones:**

- `M05.1` — **Tier model** (`done_or_near_done`): Admission/planning/dispatch/barrier/execution/observation/projection/closure as authority stages.
- `M05.2` — **Lane model** (`done_or_near_done`): Lane capacity/capability/fairness without semantic authority.
- `M05.3` — **ConstraintSpec/Provider** (`done_or_near_done`): Requirement/Claim/Lease, snapshots/epochs, Allow/Wait/Deny/Restrict decisions.
- `M05.4` — **Graph indexes** (`done_or_near_done`): wait_by_fact, wait_by_scope, active_by_write_scope, ready_by_lane, incremental rebuild.
- `M05.5` — **Safe frontier selection** (`done_or_near_done`): Ready antichain, no hard deps, no conflicts, lane/resource budgets.
- `M05.6` — **Drain/fence protocol** (`done_or_near_done`): domain/global drain epochs, safe points, disjoint domains, frozen sidecars.
- `M05.7` — **Runtime constraint updates** (`done_or_near_done`): capacity/quota/freeze/rate-limit updates with epoch, rebuild frontier, no truth rewrite.
- `M05.8` — **why-not-parallel** (`done_or_near_done`): Machine-readable blocker/rationale for concurrency decisions.

**Exit gate:** Dispatcher может объяснить `ready`, `blocked`, `why-not-parallel` и выдать safe antichain без конфликтующих mutable writes.

## S06 — AuthZ, RBAC/ABAC/ReBAC, approvals и capabilities

**Status:** `advanced_in_repo`

**Theme:** Сделать доступ частью dispatch protocol, а не middleware around endpoints.

**Milestones:**

- `M06.1` — **Authz policy model** (`done_or_near_done`): AuthzPolicy, stages, policy_id/version, freshness/expiry, default deny.
- `M06.2` — **Cedar adapter prototype** (`done_or_near_done`): Embedded PDP mapping Predicate/Subject/Context to Cedar Principal/Action/Resource/Context.
- `M06.3` — **Casbin/AuthZEN/OpenFGA sketches** (`done_or_near_done`): Adapter contracts; do not bake one engine into core.
- `M06.4` — **Approval as action** (`done_or_near_done`): gate.approve/gate.deny bound to action_id+plan_hash+impact_set_hash.
- `M06.5` — **Step-up and SoD** (`done_or_near_done`): MFA/step-up, separation-of-duties, approval freshness.
- `M06.6` — **Execution capability enforcement** (`done_or_near_done`): Worker executes only with scoped capability derived from barrier.
- `M06.7` — **Projection access/redaction** (`done_or_near_done`): Read/projection authz, sensitive-field redaction policy.

**Exit gate:** RuntimeExecution action не пересекает barrier без свежего Allow, approval bound к action_id+plan_hash+impact_set_hash, executor работает только со scoped capability.

## S07 — Observability, explainability и DX

**Status:** `advanced_in_repo`

**Theme:** Сделать систему понятной и приятной: explain(), CLI, graph export, redaction, support bundles, docs/cookbook.

**Milestones:**

- `M07.1` — **CLI explain/why** (`done_or_near_done`): explain, why-blocked, why-not-parallel, graph export, replay diagnostics.
- `M07.2` — **Graph export** (`done_or_near_done`): Mermaid/DOT/JSON graph slices with blockers/witnesses/leases.
- `M07.3` — **Tracing connector** (`done_or_near_done`): Structured action/op spans; logs derived, not truth.
- `M07.4` — **OpenTelemetry optional** (`done_or_near_done`): OTLP logs/traces/metrics adapter; fail-open for telemetry only.
- `M07.5` — **Redaction policy** (`done_or_near_done`): Audit/log/projection/replay/support-bundle redaction classes.
- `M07.6` — **Support bundle** (`done_or_near_done`): Sanitized bundle with trace, graph slice, route rationale, environment/tool report.
- `M07.7` — **Cookbook docs** (`done_or_near_done`): Add action, approval, conflict, drain, replay, adapter, authz, projection recipes.

**Exit gate:** Пользователь может понять, почему action запущен, заблокирован, не параллелится, требует approval/drain/lease, и воспроизвести это в replay.

## S08 — Runtime shell и adapters

**Status:** `active_next`

**Theme:** Подключить execution backends и persistence без превращения Causlane в workflow engine.

**Milestones:**

- `M08.1` — **In-process runtime** (`done_or_near_done`): Tokio partition loops, bounded queues, semaphores for capacity, no global lock.
- `M08.2` — **Audit adapters** (`done_or_near_done`): In-memory, SQLite, Postgres append-only audit; group commit policies.
- `M08.3` — **Executor port/adapters** (`done_or_near_done`): Tower-like Service port; hard effects only with capability.
- `M08.4` — **Apalis adapter** (`done_or_near_done`): Rust-native jobs backend; certification tests.
- `M08.5` — **Restate adapter** (`done_or_near_done`): Durable handler/workflow adapter; optional feature.
- `M08.6` — **Temporal/Dapr/Conductor adapters** (`future`): Experimental/community boundary; no hard dependency.
- `M08.7` — **Adapter certification** (`done_or_near_done`): Bounded certification matrix for existing adapters; retry/cancel/truth orchestration deferred.
- `M08.8` — **Shadow mode** (`done_or_near_done`): Runtime shadow comparer over `InProcessRuntimeEvent`; compare expected vs actual without enforcement.

**Exit gate:** Есть in-process runtime, SQL audit, tracing adapter, Cedar adapter и один job/durable backend adapter, прошедшие adapter certification.

## S09 — Performance, reliability и high-throughput readiness

**Status:** `active_next`

**Theme:** Сохранить correctness, но не превратить hot path в тяжелый workflow/control-plane overhead.

**Milestones:**

- `M09.1` — **Bench suite** (`done_or_near_done`): Criterion baseline for registry normalize, plan_hash, bundle load, replay verify, frontier conflict selection, lease grant, barrier append, explain.
- `M09.2` — **Partitioned dispatcher** (`done_or_near_done`): Host dispatch v2 partition routes and in-process admission coordinator with deterministic cross-partition ordering.
- `M09.3` — **Batched durability** (`done_or_near_done`): `AuditLogPort::append_batch` gives all-or-nothing ordered group commit over the existing audit boundary.
- `M09.4` — **Backpressure policy** (`done_or_near_done`): Runtime-local wait/fail-fast overload policy over bounded in-process partition queues.
- `M09.5` — **Plan/template caches** (`done_or_near_done`): Pure in-memory plan/template cache keyed by canonical plan material and compile snapshot refs.
- `M09.6` — **Operational SLOs** (`done_or_near_done`): Typed operational SLO measurement catalog for submit/admission/barrier/replay/explain p50/p95, queue depth and stale snapshot age.
- `M09.7` — **Chaos/recovery tests** (`done_or_near_done`): Bounded in-process chaos/recovery evidence for slow handlers, host-owned retry, provider failure, route contention and ephemeral partition restart.

**Exit gate:** Есть benchmark profile, partitioned dispatcher, bounded queues, batched durable writes и typed SLO catalog для admission/barrier/replay/explain.

## S10 — Formal depth и proof hardening

**Status:** `active_next`

**Theme:** Углубить доказательства после стабилизации ядра: больше invariants, scopes, interleavings, preservation/refinement.

**Milestones:**

- `M10.1` — **Invariant expansion** (`exists`): Shared active/planned/known invariant-id catalog and planned I-011..I-020 reservations without coverage credit.
- `M10.2` — **P interleavings depth** (`exists_expand`): P-first bounded controls for retry duplicate execution, authz revocation and constraint epoch; cancellation/partition remain planned.
- `M10.3` — **Kani integration** (`exists_expand`): cargo kani profile, unwind bounds, generated fixtures, CI/nightly split.
- `M10.4` — **Verus/Lean proof hardening** (`exists_expand`): Verus and Lean4 proof lanes have a machine-readable always-blocking contract; deeper proof semantics remain expansion work.
- `M10.5` — **Coverage anti-theatre** (`exists_expand`): Coverage matrix Markdown is fully drift-checked from the receipt-derived report; active docs point to generated coverage instead of hand-maintained live inventories.
- `M10.6` — **Proof/refinement docs** (`exists_expand`): Schema-validated proof/refinement scope classifies claim strength, and generated Markdown drift is checked by the formal gate.

**Exit gate:** Coverage matrix показывает blocking proof lanes для релизных invariants, stale proofs блокируют release, exceptions имеют expiry и owner.

## S11 — Public pre-alpha/bootstrap and alpha publication

**Status:** `active_pre_alpha_prep`

**Theme:** Prepare public source/package provenance before public alpha.

**Milestones:**

- `M11.1` — **Crate naming/publish check** (`exists_expand`): No-publish readiness cleared deterministic local blockers and handed off to staged release evidence.
- `M11.2` — **Feature flags** (`done_or_near_done`): default minimal; existing optional runtime integrations are explicit and non-default.
- `M11.3` — **Public API review** (`done_or_near_done`): Rust API guidelines, builders, newtypes, no raw Strings on critical fields.
- `M11.4` — **Examples** (`done_or_near_done`): simple-local, approval-gate, consequence-parallelism and why-not-parallel are runnable and checked.
- `M11.5` — **Security/release hygiene** (`done_or_near_done`): licenses, dependency audit, context-pack scan, secret rules, vulnerability policy.
- `M11.6` — **Contributor guide** (`done_or_near_done`): public contributor guide consolidates ADR process, new predicate checklist, formal obligation template, adapter certification and AI accountability.
- `M11.7` — **Release notes** (`done_or_near_done`): Clear limitations: not workflow engine, formal lanes coverage, unstable APIs.

**Exit gate:** For `0.0.1`, the refactor-before-publication gate records PUB0-PUB4 complete, public API review is recorded, GitHub baseline is curated/scanned, package file lists are reviewed, crates are dry-run/published in dependency order, and downstream smoke can depend on `causlane@0.0.1`. Public alpha `0.1.x` additionally requires runnable examples, usable cookbook/docs, honest receipt-backed formal/replay status and a shaped feature/facade surface.

## S12 — Beta integrations и ecosystem validation

**Status:** `active_next`

**Theme:** Проверить полезность на нескольких реальных интеграциях и улучшить DX/adapters.

**Milestones:**

- `M12.1` — **Reference integration 1** (`done_or_near_done`): Rust service with API+worker+audit+projection.
- `M12.2` — **Reference integration 2** (`done_or_near_done`): Agent/tool execution or CI/CD/release orchestration.
- `M12.3` — **Migration/shadow docs** (`done_or_near_done`): How to adopt incrementally without rewrite.
- `M12.4` — **Adapter ecosystem** (`planned`): Document external adapter interface, compatibility/certification.
- `M12.5` — **DX feedback loop** (`planned`): Simplify common paths, reduce ceremony, improve error messages.
- `M12.6` — **Semver pre-1.0 freeze plan** (`planned`): Identify APIs slated for stabilization.

**Exit gate:** Есть 2–3 reference integrations, migration/shadow-mode story и feedback-driven API hardening.

## S13 — 1.0 release readiness

**Status:** `planned`

**Theme:** Стабилизировать публичный API, semver, invariants, docs, formal/replay gates и эксплуатационные гарантии.

**Milestones:**

- `M13.1` — **API stabilization** (`planned`): Core types, bundle format, event model, replay trace, adapter traits.
- `M13.2` — **Compatibility policy** (`planned`): Schema migration, trace replay across versions, deprecation windows.
- `M13.3` — **Formal/replay release gate** (`planned`): No release unless formal-ready, replay corpus, coverage matrix, exceptions valid.
- `M13.4` — **Production readiness docs** (`planned`): Operations, performance tuning, failure modes, backup/restore audit.
- `M13.5` — **Security model docs** (`planned`): Threat model, authz defaults, capability scope, redaction, supply chain.
- `M13.6` — **1.0 release** (`planned`): Publish stable crates and docs with honest guarantees.

**Exit gate:** 1.0 может быть выпущен без обещаний, которые ядро/формальные gates/replay/adapters не подтверждают.

## S14 — Post-1.0: platform, ecosystem, advanced modes

**Status:** `future`

**Theme:** Развивать control-plane/service mode, marketplace adapters, richer proofs, dashboards, distributed deployment.

**Milestones:**

- `M14.1` — **Service/control-plane mode** (`future`): Optional server mode; still small kernel authority.
- `M14.2` — **Dashboard/UI** (`future`): Visual graph, blockers, audit/replay exploration.
- `M14.3` — **Advanced distributed coordination** (`future`): Multi-node scheduler, HA audit, failover only after formal model/design.
- `M14.4` — **Adapter marketplace/catalog** (`future`): Community adapters with certification receipts.
- `M14.5` — **Advanced proof/language support** (`future`): More proof generators/languages as generated projections, not manual authority.

**Exit gate:** Post-1.0 расширения не ломают small-core principle и не превращают проект в скрытый workflow engine.
