# Causlane product development track — full bundle


---

<!-- README.md -->

# Causlane Product Development Track

Дата сборки: 2026-06-21

Этот пакет описывает полный трек развития продукта **Causlane**: от текущего contract/formal scaffold до public alpha, beta, 1.0 и post-1.0 направлений.

Causlane — это Rust-first portable semantic dispatch kernel for typed, auditable, replayable, consequence-aware actions. Проект не должен становиться workflow engine, job queue или distributed scheduler. Его задача — дать маленький проверяемый control layer: `ActionCall -> ActionPlan -> dispatch/barrier/leases/witnesses -> execution -> observed truth -> projection/replay/explainability`.

## Что внутри

```text
00-executive-roadmap.md               единое краткое описание трека
01-product-track-map.md               все stages и gates
02-milestone-catalog.md               каталог milestone-ов
03-technical-decisions.md             основные технические решения
04-readiness-gates.md                 quality/formal/release gates
05-dependency-map.md                  зависимости между stage-ами
06-risk-register.md                   ключевые риски и меры
07-service-workstream.md              cleanup/tooling/settings/CI/security work
08-formal-verification-track.md       Alloy/P/Kani/Verus/Lean4 трек
09-runtime-adapter-track.md           runtime/adapters трек
10-release-strategy.md                alpha/beta/1.0 стратегия
roadmap.yaml                          machine-readable версия roadmap
stages/*.md                           stage-by-stage документы
milestones/*.md                       milestone-by-milestone документы
templates/*.md                        шаблоны для новых milestone/stage/status updates
```

## Основной принцип

```text
Docs define the contract.
Formal models attack the contract.
Replay executes the contract.
Rust kernel enforces the contract.
Adapters spend the contract.
Audit records the truth.
Projections explain the truth.
```

## Как читать

1. Начать с `00-executive-roadmap.md`.
2. Затем открыть `01-product-track-map.md`.
3. Для планирования sprint/release использовать `02-milestone-catalog.md` и `04-readiness-gates.md`.
4. Для архитектурной преемственности использовать `03-technical-decisions.md`.
5. Для формальных моделей использовать `08-formal-verification-track.md`.

## Repository integration note

`causlane_product_track_full_ru.md` is a concatenated reference bundle. Edit the
atomic files in this directory (`00-*`, `stages/*`, `milestones/*`,
`templates/*`, `roadmap.*`) as the primary documentation sources.

This product track is a planning corpus. It does not replace machine-derived
formal status, receipts, stale-check, coverage reports, or release gate outputs.


---

<!-- 00-executive-roadmap.md -->

# 00. Executive roadmap

## Назначение продукта

Causlane — библиотека/фреймворк для typed, auditable, replayable, consequence-aware action dispatch. Она нужна там, где одно значимое действие иначе распадается на UI button, API endpoint, CLI command, job type, workflow step, policy rule, telemetry event и worker handler.

Целевое обещание:

```text
Define actions once. Enforce lifecycle, barriers, witnesses, leases, authz, audit, replay, and explainability everywhere.
```

## Почему не workflow engine

Causlane не берет на себя durable execution как продуктовую платформу. Она задает semantic/control protocol и интегрируется с Tokio, Apalis, Restate, Temporal, Dapr, Conductor, SQL audit, Cedar/AuthZEN/OpenFGA и т.д. через adapters.

Красная линия:

```text
Causlane decides whether an action is allowed to cross a semantic/lifecycle boundary.
Runtime adapters decide how to physically execute already-authorized work.
```

## Текущая исходная точка

По состоянию текущего репозитория уже есть:

- docs-first architecture;
- ADR chain;
- RegistryManifest → CompiledDispatchBundle;
- BundleHash / PlanHash / canonical serialization;
- formal IR;
- generated Alloy/P/Kani/Verus/Lean4 contour;
- receipts/stale-check/coverage derivation;
- replay oracle and negative controls;
- formal doctor/install/smoke tooling;
- hexagonal crate layout.

Формальные модели v1 уже образуют рабочий verification contour (generated artifacts + receipts + stale-check + negative controls, `formal-verify-all` зелёный под default-профилем). Reference kernel (S03), сценарии/replay/contract testing (S04), constraint plane / frontier scheduler (S05) и AuthZ/approval/capability plane (S06) закреплены — `KernelContracts` единая runtime/replay authority; tiers/lanes, constraints/frontier, drain/fence, why-not-parallel, authz policy, approvals, scoped execution capabilities и projection redaction кодифицированы чистыми модулями с tests/replay coverage. Текущий активный этап — Observability, explainability и DX (S07); затем runtime/adapters, потом alpha/beta/1.0.

## Product stages

| Stage | Название | Статус | Назначение |
|---|---|---|---|
| S00 | Контекст, назначение и продуктовая рамка | baseline_exists | Зафиксировать смысл продукта, non-goals, целевую аудиторию, язык и границы. |
| S01 | Контрактный фундамент и formal readiness | advanced_in_repo | Стабилизировать compiled bundle, canonical hash, scenario/replay, formal IR, receipts и stale-check. |
| S02 | Формальные модели v1 | advanced_in_repo | Сделать Alloy/P/Kani/Verus/Lean4 не демонстрацией, а рабочим verification contour для ключевых инвариантов. |
| S03 | Reference kernel и executable semantics | advanced_in_repo | Превратить контракт в маленькое чистое ядро: lifecycle reducer, guards, frontier abstractions, replay oracle. |
| S04 | Сценарии, replay и contract testing | advanced_in_repo | Расширить executable scenario catalog и replay oracle до уровня основного devtool. |
| S05 | Constraint Plane, graph/frontier и lanes/tiers | advanced_in_repo | Реализовать безопасную автоматическую диспетчеризацию параллельности на основе consequences, claims, leases, witnesses и constraints. |
| S06 | AuthZ, RBAC/ABAC/ReBAC, approvals и capabilities | advanced_in_repo | Сделать доступ частью dispatch protocol, а не middleware around endpoints. |
| S07 | Observability, explainability и DX | active_next | Сделать систему понятной и приятной: explain(), CLI, graph export, redaction, support bundles, docs/cookbook. |
| S08 | Runtime shell и adapters | active_next | Подключить execution backends и persistence без превращения Causlane в workflow engine. |
| S09 | Performance, reliability и high-throughput readiness | active_next | Сохранить correctness, но не превратить hot path в тяжелый workflow/control-plane overhead. |
| S10 | Formal depth и proof hardening | planned | Углубить доказательства после стабилизации ядра: больше invariants, scopes, interleavings, preservation/refinement. |
| S11 | Public pre-alpha/bootstrap and alpha publication | active_pre_alpha_prep | Prepare public source/package provenance before public alpha. |
| S12 | Beta integrations и ecosystem validation | planned | Проверить полезность на нескольких реальных интеграциях и улучшить DX/adapters. |
| S13 | 1.0 release readiness | planned | Стабилизировать публичный API, semver, invariants, docs, formal/replay gates и эксплуатационные гарантии. |
| S14 | Post-1.0: platform, ecosystem, advanced modes | future | Развивать control-plane/service mode, marketplace adapters, richer proofs, dashboards, distributed deployment. |


## Главные release criteria

### Alpha

Alpha можно выпускать, когда:

- core API компилируется и имеет минимально удобную facade;
- есть runnable examples;
- formal-ready/formal-verify-all проходят;
- replay oracle покрывает основные valid/invalid сценарии;
- docs честно описывают limitations;
- adapters не обещают production HA.

### Beta

Beta можно выпускать, когда:

- есть 2–3 реальные интеграции;
- API и bundle format стабилизируются;
- adapter certification работает;
- performance/backpressure story измерена;
- migration/shadow-mode documented.

### 1.0

1.0 можно выпускать, когда:

- stable public API;
- compatibility/migration policy;
- release gates enforce docs/formal/replay/coverage;
- security model and operational docs complete;
- project no longer relies on hand-maintained formal claims.


---

<!-- 01-product-track-map.md -->

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

**Status:** `active_next`

**Theme:** Расширить executable scenario catalog и replay oracle до уровня основного devtool.

**Milestones:**

- `M04.1` — **Golden success scenarios** (`planned`): release_promote, approval, projection, conflict-free parallelism, read-only sidecar.
- `M04.2` — **Negative scenario suite** (`planned`): execution_without_barrier, observed_without_execution, projection_without_anchor, wrong plan, missing witness, conflicting leases.
- `M04.3` — **Scenario-to-trace compiler** (`planned`): One scenario source emits trace, formal facts, replay expectation.
- `M04.4` — **Replay explain output** (`planned`): Replay failures return exact invariant/error and causal location.
- `M04.5` — **Contract testing harness** (`planned`): dispatch_test! / YAML runner for predicates and adapters.
- `M04.6` — **Mutation/fuzz tests** (`planned`): Generate malformed traces and small worlds; ensure fail-closed.

**Exit gate:** Каждое новое predicate/feature требует scenario + replay expectation + formal obligation/exemption.

## S05 — Constraint Plane, graph/frontier и lanes/tiers

**Status:** `active_next`

**Theme:** Реализовать безопасную автоматическую диспетчеризацию параллельности на основе consequences, claims, leases, witnesses и constraints.

**Milestones:**

- `M05.1` — **Tier model** (`planned`): Admission/planning/dispatch/barrier/execution/observation/projection/closure as authority stages.
- `M05.2` — **Lane model** (`planned`): Lane capacity/capability/fairness without semantic authority.
- `M05.3` — **ConstraintSpec/Provider** (`planned`): Requirement/Claim/Lease, snapshots/epochs, Allow/Wait/Deny/Restrict decisions.
- `M05.4` — **Graph indexes** (`planned`): wait_by_fact, wait_by_scope, active_by_write_scope, ready_by_lane, incremental rebuild.
- `M05.5` — **Safe frontier selection** (`planned`): Ready antichain, no hard deps, no conflicts, lane/resource budgets.
- `M05.6` — **Drain/fence protocol** (`planned`): domain/global drain epochs, safe points, disjoint domains, frozen sidecars.
- `M05.7` — **Runtime constraint updates** (`planned`): capacity/quota/freeze/rate-limit updates with epoch, rebuild frontier, no truth rewrite.
- `M05.8` — **why-not-parallel** (`planned`): Machine-readable blocker/rationale for concurrency decisions.

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

**Status:** `active_next`

**Theme:** Сделать систему понятной и приятной: explain(), CLI, graph export, redaction, support bundles, docs/cookbook.

**Milestones:**

- `M07.1` — **CLI explain/why** (`done_or_near_done`): explain, why-blocked, why-not-parallel, graph export, replay diagnostics.
- `M07.2` — **Graph export** (`done_or_near_done`): Mermaid/DOT/JSON graph slices with blockers/witnesses/leases.
- `M07.3` — **Tracing connector** (`done_or_near_done`): Structured action/op spans; logs derived, not truth.
- `M07.4` — **OpenTelemetry optional** (`done_or_near_done`): OTLP logs/traces/metrics adapter; fail-open for telemetry only.
- `M07.5` — **Redaction policy** (`done_or_near_done`): Audit/log/projection/replay/support-bundle redaction classes.
- `M07.6` — **Support bundle** (`done_or_near_done`): Sanitized bundle with trace, graph slice, route rationale, environment/tool report.
- `M07.7` — **Cookbook docs** (`planned`): Add action, approval, conflict, drain, replay, adapter, authz, projection recipes.

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
- `M09.7` — **Chaos/recovery tests** (`planned`): Audit slow, worker retry, provider unavailable, drain under load, partition restart.

**Exit gate:** Есть benchmark profile, partitioned dispatcher, bounded queues, batched durable writes и typed SLO catalog для admission/barrier/replay/explain.

## S10 — Formal depth и proof hardening

**Status:** `planned`

**Theme:** Углубить доказательства после стабилизации ядра: больше invariants, scopes, interleavings, preservation/refinement.

**Milestones:**

- `M10.1` — **Invariant expansion** (`planned`): I-001..I-010 to I-020+ for cancellation, idempotency, drains, stale authz, overlay monotonicity.
- `M10.2` — **P interleavings depth** (`planned`): Bounded race models for retry, cancellation, authz revocation, constraint epoch, worker partition.
- `M10.3` — **Kani integration** (`planned`): cargo kani profile, unwind bounds, generated fixtures, CI/nightly split.
- `M10.4` — **Verus/Lean proof hardening** (`exists_expand`): Verus and Lean4 proof lanes have a machine-readable always-blocking contract; deeper proof semantics remain expansion work.
- `M10.5` — **Coverage anti-theatre** (`exists_expand`): Coverage matrix Markdown is fully drift-checked from the receipt-derived report; active docs point to generated coverage instead of hand-maintained live inventories.
- `M10.6` — **Proof/refinement docs** (`exists_expand`): Schema-validated proof/refinement scope classifies claim strength, and generated Markdown drift is checked by the formal gate.

**Exit gate:** Coverage matrix показывает blocking proof lanes для релизных invariants, stale proofs блокируют release, exceptions имеют expiry и owner.

## S11 — Public pre-alpha/bootstrap and alpha publication

**Status:** `active_pre_alpha_prep`

**Theme:** Prepare public source/package provenance before public alpha.

## Current sub-track: publication bootstrap

The immediate next action is not upload and not GitHub opening. It is the full
publication refactor and readiness hardening sequence. PUB5 is unreachable until
PUB0-PUB4 are complete:

```text
PUB0 refactor first
PUB1 readability/maintainability
PUB2 public API review
PUB3 human/agent docs
PUB4 GitHub history/repository preparation
PUB5 staged crates.io publication
PUB6 post-publication stabilization
```

**Milestones:**

- `M11.1` — **Crate naming/publish check** (`exists_expand`): No-publish readiness clears deterministic local blockers while publication execution remains deferred.
- `M11.2` — **Feature flags** (`done_or_near_done`): default minimal; existing optional runtime integrations are explicit and non-default.
- `M11.3` — **Public API review** (`planned`): Rust API guidelines, builders, newtypes, no raw Strings on critical fields.
- `M11.4` — **Examples** (`planned`): simple-local, approval-gate, consequence-parallelism, why-not-parallel.
- `M11.5` — **Security/release hygiene** (`planned`): licenses, dependency audit, context-pack scan, secret rules, vulnerability policy.
- `M11.6` — **Contributor guide** (`planned`): ADR process, new predicate checklist, formal obligation template, adapter certification.
- `M11.7` — **Release notes** (`planned`): Clear limitations: not workflow engine, formal lanes coverage, unstable APIs.

**Exit gate:** For `0.0.1`, the refactor-before-publication gate records PUB0-PUB4 complete, public API review is recorded, GitHub baseline is curated/scanned, package file lists are reviewed, crates are dry-run/published in dependency order, and downstream smoke can depend on `causlane@0.0.1`. Public alpha `0.1.x` additionally requires runnable examples, usable cookbook/docs, honest receipt-backed formal/replay status and a shaped feature/facade surface.

## S12 — Beta integrations и ecosystem validation

**Status:** `planned`

**Theme:** Проверить полезность на нескольких реальных интеграциях и улучшить DX/adapters.

**Milestones:**

- `M12.1` — **Reference integration 1** (`planned`): Rust service with API+worker+audit+projection.
- `M12.2` — **Reference integration 2** (`planned`): Agent/tool execution or CI/CD/release orchestration.
- `M12.3` — **Migration/shadow docs** (`planned`): How to adopt incrementally without rewrite.
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


---

<!-- 02-milestone-catalog.md -->

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
- **Status:** `planned`
- **Purpose:** One scenario source emits trace, formal facts, replay expectation.

## M04.4 — Replay explain output

- **Stage:** S04
- **Status:** `planned`
- **Purpose:** Replay failures return exact invariant/error and causal location.

## M04.5 — Contract testing harness

- **Stage:** S04
- **Status:** `planned`
- **Purpose:** dispatch_test! / YAML runner for predicates and adapters.

## M04.6 — Mutation/fuzz tests

- **Stage:** S04
- **Status:** `planned`
- **Purpose:** Generate malformed traces and small worlds; ensure fail-closed.

## M05.1 — Tier model

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** Admission/planning/dispatch/barrier/execution/observation/projection/closure as authority stages.

## M05.2 — Lane model

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** Lane capacity/capability/fairness without semantic authority.

## M05.3 — ConstraintSpec/Provider

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** Requirement/Claim/Lease, snapshots/epochs, Allow/Wait/Deny/Restrict decisions.

## M05.4 — Graph indexes

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** wait_by_fact, wait_by_scope, active_by_write_scope, ready_by_lane, incremental rebuild.

## M05.5 — Safe frontier selection

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** Ready antichain, no hard deps, no conflicts, lane/resource budgets.

## M05.6 — Drain/fence protocol

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** domain/global drain epochs, safe points, disjoint domains, frozen sidecars.

## M05.7 — Runtime constraint updates

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** capacity/quota/freeze/rate-limit updates with epoch, rebuild frontier, no truth rewrite.

## M05.8 — why-not-parallel

- **Stage:** S05
- **Status:** `planned`
- **Purpose:** Machine-readable blocker/rationale for concurrency decisions.

## M06.1 — Authz policy model

- **Stage:** S06
- **Status:** `planned`
- **Purpose:** AuthzPolicy, stages, policy_id/version, freshness/expiry, default deny.

## M06.2 — Cedar adapter prototype

- **Stage:** S06
- **Status:** `planned`
- **Purpose:** Embedded PDP mapping Predicate/Subject/Context to Cedar Principal/Action/Resource/Context.

## M06.3 — Casbin/AuthZEN/OpenFGA sketches

- **Stage:** S06
- **Status:** `planned`
- **Purpose:** Adapter contracts; do not bake one engine into core.

## M06.4 — Approval as action

- **Stage:** S06
- **Status:** `planned`
- **Purpose:** gate.approve/gate.deny bound to action_id+plan_hash+impact_set_hash.

## M06.5 — Step-up and SoD

- **Stage:** S06
- **Status:** `planned`
- **Purpose:** MFA/step-up, separation-of-duties, approval freshness.

## M06.6 — Execution capability enforcement

- **Stage:** S06
- **Status:** `planned`
- **Purpose:** Worker executes only with scoped capability derived from barrier.

## M06.7 — Projection access/redaction

- **Stage:** S06
- **Status:** `planned`
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
- **Status:** `planned`
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
- **Status:** `planned`
- **Purpose:** Audit slow, worker retry, provider unavailable, drain under load, partition restart.

## M10.1 — Invariant expansion

- **Stage:** S10
- **Status:** `planned`
- **Purpose:** I-001..I-010 to I-020+ for cancellation, idempotency, drains, stale authz, overlay monotonicity.

## M10.2 — P interleavings depth

- **Stage:** S10
- **Status:** `planned`
- **Purpose:** Bounded race models for retry, cancellation, authz revocation, constraint epoch, worker partition.

## M10.3 — Kani integration

- **Stage:** S10
- **Status:** `planned`
- **Purpose:** cargo kani profile, unwind bounds, generated fixtures, CI/nightly split.

## M10.4 — Verus/Lean proof hardening

- **Stage:** S10
- **Status:** `exists_expand`
- **Purpose:** Machine-enforced blocking contract for Verus/Lean proof lanes.

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
- **Purpose:** No-publish readiness clears local facade blockers while publication execution remains deferred.

## M11.2 — Feature flags

- **Stage:** S11
- **Status:** `done_or_near_done`
- **Purpose:** default minimal; existing optional runtime integrations are explicit and non-default.

## M11.3 — Public API review

- **Stage:** S11
- **Status:** `planned`
- **Purpose:** Rust API guidelines, builders, newtypes, no raw Strings on critical fields.

## M11.4 — Examples

- **Stage:** S11
- **Status:** `planned`
- **Purpose:** simple-local, approval-gate, consequence-parallelism, why-not-parallel.

## M11.5 — Security/release hygiene

- **Stage:** S11
- **Status:** `planned`
- **Purpose:** licenses, dependency audit, context-pack scan, secret rules, vulnerability policy.

## M11.6 — Contributor guide

- **Stage:** S11
- **Status:** `planned`
- **Purpose:** ADR process, new predicate checklist, formal obligation template, adapter certification.

## M11.7 — Release notes

- **Stage:** S11
- **Status:** `planned`
- **Purpose:** Clear limitations: not workflow engine, formal lanes coverage, unstable APIs.

## M12.1 — Reference integration 1

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** Rust service with API+worker+audit+projection.

## M12.2 — Reference integration 2

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** Agent/tool execution or CI/CD/release orchestration.

## M12.3 — Migration/shadow docs

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** How to adopt incrementally without rewrite.

## M12.4 — Adapter ecosystem

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** Document external adapter interface, compatibility/certification.

## M12.5 — DX feedback loop

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** Simplify common paths, reduce ceremony, improve error messages.

## M12.6 — Semver pre-1.0 freeze plan

- **Stage:** S12
- **Status:** `planned`
- **Purpose:** Identify APIs slated for stabilization.

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


---

<!-- 03-technical-decisions.md -->

# 03. Основные технические решения

## TD-001. Causlane — semantic control layer, не workflow engine

**Решение:** не строить замену Temporal/Restate/Dapr/Conductor/Apalis/Fang. Строить маленькое semantic dispatch ядро и adapters.

**Почему:** durable workflows, job queues, retries, timers, worker pools и HA schedulers уже существуют. Уникальная ценность Causlane — typed action semantics, consequence profiles, barriers, witnesses, leases, replay and explainability.

## TD-002. Docs-first, formal-first, code-after-contract

**Решение:** продукт развивается от normative docs и executable contracts к formal projections и только потом к runtime/adapters.

**Почему:** основная сложность — semantic/lifecycle correctness, а не техника исполнения. Если код появляется раньше контракта, handler/job/endpoint снова становятся семантикой.

## TD-003. Generate-don't-maintain

**Решение:** формальные модели, replay expectations и coverage строятся из compiled bundle/formal IR, а не поддерживаются вручную.

**Почему:** ручная Alloy/P/Verus модель быстро становится second semantic center. Proof должен атаковать тот же contract, который исполняет runtime.

## TD-004. Single observed truth authority

**Решение:** audit/event journal — единственный авторитет observed truth. Logs, metrics, UI, execution graph, status pages — derived projections.

**Почему:** иначе replay, расследования и formal checks расходятся с реальным runtime.

## TD-005. `predicate(subject, circumstance)` как canonical action shape

**Решение:** surface commands нормализуются в typed ActionCall с predicate, subject, circumstance, actor/origin, source surface, correlation/idempotency.

**Почему:** endpoint/button/job type не должны владеть meaning.

## TD-006. Planning is pure

**Решение:** `compile(ActionCall, typed snapshots) -> ActionPlan` без I/O, wall-clock, RNG, hidden mutable state.

**Почему:** иначе plan_hash/replay/formal reasoning не работают.

## TD-007. PlanHash через canonical PlanHashMaterial

**Решение:** `plan_hash = sha256(canonical_plan_hash_material)`, где material содержит predicate/version, action identity, bundle hash, planner fingerprint, subject/circumstance fingerprints, ops, planned impacts, required witnesses/claims, policies.

**Почему:** approval, replay и stale detection должны быть привязаны к точному плану, а не к строковой заглушке.

## TD-008. Anchor отдельно от witnesses

**Решение:** projection имеет typed `TruthAnchor`; witnesses используются для causal/guard evidence. Anchor может также быть witness, но это отдельное поле.

**Почему:** witness отвечает “почему переход разрешен”, anchor — “из какой observed truth построена projection”.

## TD-009. Authz deny by default

**Решение:** unknown predicate, missing authz policy, missing/expired/stale decision для RuntimeExecution fail closed. Explicit public/dev mode нужен явно.

**Почему:** authorization — часть dispatch protocol, а не middleware; безопасный default должен быть deny.

## TD-010. Merge protocol explicit and verified

**Решение:** пересекающиеся mutable write scopes конфликтуют по умолчанию. `mergeable(a,b)` true только при explicit bundle-level `MergeProtocolSpec` со статусом verified/enabled.

**Почему:** нельзя скрыто разрешать параллельные mutable writes строкой в Op.

## TD-011. Tiers vs lanes

**Решение:** tiers — authority/lifecycle stages; lanes — capacity/capability/fairness slots внутри tier. Lane не может менять semantic obligations.

**Почему:** иначе появляется “fast lane without barrier”.

## TD-012. Constraint Plane отдельно от dispatcher kernel

**Решение:** constraints modeled as Requirements, Claims, Leases, Snapshots, Decisions. Providers дают domain/resource/business state, dispatcher enforces through leases/barriers.

**Почему:** библиотека не знает бизнес-ресурсы, но должна enforce constraints uniformly.

## TD-013. Execution barrier как permission boundary

**Решение:** hard side effect может стартовать только после durable barrier, содержащего witnesses, leases, impact_set_hash, authz refs, constraint snapshot.

**Почему:** архитектура должна заранее зафиксировать право на side effect, а не объяснять изменения задним числом.

## TD-014. Replay as executable oracle

**Решение:** replay проверяет traces against bundle/formal IR and fails closed on missing barrier, missing execution, missing anchor, plan mismatch, lease conflict, authz defects, lifecycle violations.

**Почему:** replay связывает runtime и formal contour.

## TD-015. Formal tool roles

```text
Alloy  -> relational/counterexample shapes.
P      -> protocol/interleaving histories.
Kani   -> bounded Rust implementation checks.
Verus  -> abstract preservation/refinement proofs.
Lean4  -> generated theorem applications/proof facet.
```

## TD-016. Hexagonal architecture

**Решение:** domain/core has no runtime dependencies; adapters sit outside. `causlane-core`, `causlane-contracts`, `causlane-replay`, `causlane-codegen`, `causlane-runtime`, `causlane-cli`, facade crate.

**Почему:** portability, testability, formalization and no accidental runtime authority in adapters.

## TD-017. Observability derived, not truth

**Решение:** tracing/OpenTelemetry/logging connectors emit derived events through the typed audit-event projection. Audit failures block hard effects; telemetry failures do not.

## TD-018. Performance design

**Решение:** hot path uses compiled IDs, indexed conflict checks, bounded queues, partition loops, batched audit writes, lazy explain; heavy formal/explain/replay stays off hot path.


---

<!-- 04-readiness-gates.md -->

# 04. Readiness gates

## Gate classes

| Gate | Назначение | Блокирует |
|---|---|---|
| Contract gate | Bundle/schema/event/replay consistency | Formal models, runtime |
| Formal gate | Generated artifacts + receipts + stale-check | Release/profile depending on policy |
| Replay gate | Positive/negative scenario corpus | Runtime/adapters/release |
| Kernel gate | Core invariants in unit/proptest/Kani | Runtime/adapters |
| Adapter gate | Certification against protocol | Adapter publication |
| Security gate | Authz defaults, redaction, context hygiene, dependency checks | Alpha/Beta/1.0 |
| Performance gate | Benchmarks/backpressure/SLOs | Runtime alpha/beta |
| Documentation gate | Docs match machine-derived status | Public release |

## Stage exit gates

### S00 — Контекст, назначение и продуктовая рамка

Команда может одним документом объяснить, что такое Causlane, чем он не является, и почему он не конкурирует с workflow/job engines.

### S01 — Контрактный фундамент и formal readiness

`just formal-ready` и `just formal-verify-all` проходят в чистой среде; coverage matrix выводится из receipts, а не из prose.

### S02 — Формальные модели v1

Для I-001/I-002/I-003/I-006/I-008/I-009 есть generated artifact + tool receipt + stale-check + negative controls.

### S03 — Reference kernel и executable semantics

Core reducer и validators можно использовать без runtime/adapters; все protocol-critical функции покрыты unit/property/Kani checks.

### S04 — Сценарии, replay и contract testing

Каждое новое predicate/feature требует scenario + replay expectation + formal obligation/exemption.

### S05 — Constraint Plane, graph/frontier и lanes/tiers

Dispatcher может объяснить `ready`, `blocked`, `why-not-parallel` и выдать safe antichain без конфликтующих mutable writes.

### S06 — AuthZ, RBAC/ABAC/ReBAC, approvals и capabilities

RuntimeExecution action не пересекает barrier без свежего Allow, approval bound к action_id+plan_hash+impact_set_hash, executor работает только со scoped capability.

### S07 — Observability, explainability и DX

Пользователь может понять, почему action запущен, заблокирован, не параллелится, требует approval/drain/lease, и воспроизвести это в replay.

### S08 — Runtime shell и adapters

Есть in-process runtime, SQL audit, tracing adapter, Cedar adapter и один job/durable backend adapter, прошедшие adapter certification.

### S09 — Performance, reliability и high-throughput readiness

Есть benchmark profile, partitioned dispatcher, bounded queues, batched durable writes и typed SLO catalog для admission/barrier/replay/explain.

### S10 — Formal depth и proof hardening

Coverage matrix показывает blocking proof lanes для релизных invariants, stale proofs блокируют release, exceptions имеют expiry и owner.

### S11 — Public pre-alpha/bootstrap and alpha publication

For `0.0.1`, the refactor-before-publication gate records PUB0-PUB4 complete,
public API review is recorded, GitHub baseline is curated/scanned, package file
lists are reviewed, crates are dry-run/published in dependency order, and
downstream smoke can depend on `causlane@0.0.1`. Public alpha `0.1.x`
additionally requires runnable examples, usable cookbook/docs, honest
receipt-backed formal/replay status and a shaped feature/facade surface.

### S12 — Beta integrations и ecosystem validation

Есть 2–3 reference integrations, migration/shadow-mode story и feedback-driven API hardening.

### S13 — 1.0 release readiness

1.0 может быть выпущен без обещаний, которые ядро/формальные gates/replay/adapters не подтверждают.

### S14 — Post-1.0: platform, ecosystem, advanced modes

Post-1.0 расширения не ломают small-core principle и не превращают проект в скрытый workflow engine.


## Minimal current formal gate

```bash
just formal-ready
just formal-verify-all
tools/coverage-matrix --check
tools/formal-exceptions-check
```

A formal claim is valid only if the lane has:

1. generated artifact;
2. source bundle hash;
3. formal IR/scenario hash where applicable;
4. concrete check_id/obligation;
5. tool-run receipt with real exit code;
6. stale-check pass;
7. coverage report derived from receipts.

## Adapter certification gate

Adapter publication uses a certification matrix, not prose. The current M08.7
bounded matrix must prove/test-demonstrate for existing adapters:

- no execution before barrier;
- executor requires scoped capability;
- audit append failure fail-closed for hard effects;
- telemetry/logging not authority.

Release/production certification additionally requires first-class evidence for:

- retry/idempotency safe for hard effects;
- observed truth commit after execution;
- projection from anchor;
- cancellation/supersession semantics.


---

<!-- 05-dependency-map.md -->

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


---

<!-- 06-risk-register.md -->

# 06. Risk register

| Risk | Why it matters | Mitigation |
|---|---|---|
| Formal model as second truth | Proofs may verify a hand-maintained fantasy, not runtime contracts | Generate artifacts from bundle/formal IR; stale-check; receipt-bound coverage |
| Overbuilding workflow engine | Causlane competes with mature runtimes and loses focus | Keep adapters outside core; document non-goals; adapter certification |
| Hot path overhead | Users reject system if every request pays full formal/control cost | Compiled bundle, partitions, indexes, bounded queues, batching, lazy explain |
| Authz bypass | Endpoint/job middleware may bypass semantic action policy | Authz stages in dispatch; default deny; scoped capabilities |
| Projection as truth | UI/status diverges from observed reality | Typed TruthAnchor; replay/formal checks |
| Merge protocol ambiguity | Parallel writes corrupt state | Default no merge; verified explicit MergeProtocolSpec only |
| Toolchain friction | Formal stack too hard to reproduce | formal-doctor/install, pinned versions, receipts, profiles |
| Proof theatre | Docs overclaim formal coverage | Coverage generated from receipts; exceptions policy; anti-overclaim checks |
| Stringly protocol fields | Invalid IDs/scopes/hash slip through | Validated newtypes, canonical serialization, schema validation |
| Adapter bypass | Adapter executes without barrier/capability | Adapter certification; guarded executor APIs |
| Documentation drift | Docs become stale as codegen changes | Machine-derived status docs; docs gate |
| Scope creep | Causlane tries to solve policy/db/observability/jobs | Non-goals, modular feature flags, external adapters |


---

<!-- 07-service-workstream.md -->

# 07. Service workstream: cleanup, settings, tooling, hygiene

This workstream is not secondary. It prevents the project from becoming unverifiable or unusable.

## Repository and toolchain

- Keep `.devinfra/tool-versions.json`, `rust-toolchain.toml`, `justfile`, formal installer and doctor aligned.
- `tools/formal-doctor --json` must be usable before Rust/Cargo availability where possible.
- Pin versions and checksums for Alloy, Z3, Kani, Verus, Lean4/P where feasible.
- Split profiles: `base`, `rust`, `ci`, `formal`, `proof`, `all`.

## Generated artifacts

- Generated files must include source bundle hash, formal IR hash, scenario hash, generator version, artifact hash.
- Generated artifacts must be ignored/committed according to explicit policy, not ad hoc.
- Stale-check must block formal coverage claims.

## Cleanup and anti-drift

- Remove obsolete readiness docs or mark superseded with date and authority pointer.
- Keep roadmap status machine-derived where possible.
- Keep ADR index updated.
- Keep examples runnable or explicitly marked placeholder.
- Delete exploratory sketches when replaced by generated artifacts, or move to `sketches/`.

## Context-pack hygiene

- Never include secrets, tokens, private keys, local paths with sensitive info, large generated outputs unless intentional.
- Run context scan before sharing.
- Redact raw policy claims/identity payloads in support bundles.

## CI lanes

- Fast PR: fmt/check/test, replay small corpus, formal stale-check, docs links.
- Formal PR: generated Alloy/P/Kani smoke, receipts, coverage matrix.
- Nightly: larger scopes/unwind/interleavings, benchmark suite, fuzz/mutation tests.
- Release: proof/all profile, adapter certification, docs/security gates.

## Documentation cleanup cadence

Every stage exit includes:

- update `docs/formal-readiness-status.md` or successor;
- update coverage matrix;
- update roadmap status;
- update ADR if decision changed;
- update examples/readme if user-facing behavior changed.


---

<!-- 08-formal-verification-track.md -->

# 08. Formal verification track

## Roles

```text
Alloy  -> relational bad worlds.
P      -> message/order/interleaving bad histories.
Kani   -> bounded checks of Rust reducers/validators.
Verus  -> abstract preservation/refinement proofs.
Lean4  -> optional generated theorem applications / proof facet.
Replay -> executable oracle over real traces.
```

## v1 invariant scope

Blocking first set:

- I-001 no execution without barrier;
- I-002 no observed truth without execution;
- I-003 no projection without observed-truth anchor;
- I-006 no conflicting mutable frontier without merge protocol;
- I-008 replay accepts only valid protocol traces;
- I-009 approval/witness exact binding.

Second set:

- I-004 overlay cannot weaken obligations;
- I-005 route derives from consequence profile;
- I-007 drain acquired only after prior overlapping leases clear;
- I-010 constraint updates affect future frontier, not past truth.

## Alloy v1

Inputs:

- Formal IR generated from bundle/scenario;
- generic core model;
- generated facts and checks;
- negative controls.

Assertions:

- NoExecutionWithoutBarrier;
- NoObservedWithoutExecution;
- NoProjectionWithoutAnchor;
- NoConflictingMutableFrontier;
- WitnessFactGrounded;
- AnchorFactGrounded;
- ApprovalBoundToPlanImpact;
- RouteHasConsequenceProfile.

## P v1

Machines/monitors:

- Dispatcher, AuditLog, LeaseManager, Worker, ConstraintProvider, ProjectionBuilder;
- NoExecutionBeforeBarrier;
- NoProjectionBeforeObservedAnchor;
- DrainBlocksNewMutableAdmission;
- NoDuplicateHardExecutionOnRetry;
- AuthzAllowBeforeBarrier.

## Kani v1

Harnesses:

- reduce_lifecycle;
- Replay accepts => trace protocol properties;
- LeaseTable no overlapping exclusive writes;
- ExecutionCapability derives only from valid barrier;
- parser/decoder fail-closed/no panic;
- quota/capacity no underflow/overflow.

## Verus v1

Proofs:

- lifecycle preservation;
- barrier-before-execution theorem;
- projection-anchor theorem;
- overlay monotonicity;
- lease map preservation;
- replay_accepts => valid_trace for abstract model.

## Lean4 v1

Keep generated and narrow. Lean4 is checked in the full formal gate; broader theorem applications still need generated artifacts, receipts and coverage rows before they are claimed.

## Formal anti-theatre rules

- No lane can claim coverage without concrete check_id and fresh receipt.
- Non-blocking proof facet cannot roll up to covered unless release profile says so.
- Negative controls must fail for expected reason.
- Exceptions must have expiry and owner.


---

<!-- 09-runtime-adapter-track.md -->

# 09. Runtime and adapter track

## Principle

Runtime adapters spend the contract; they do not create semantics.

## Core runtime shell

- Tokio partition loops by primary conflict domain / tenant / subject.
- Bounded queues for ingress/planning/barrier/execution/projection/observability.
- Lane semaphores for capacity; durable leases for correctness.
- Group commit for audit/barrier where allowed.
- Guarded executor API only; no raw execute for hard effects.

## Persistence adapters

- In-memory for tests and examples.
- SQLite local/dev.
- Postgres production/server mode.
- Append-only audit first; event store/CQRS framework optional later.

## Execution adapters

- In-process executor first.
- Apalis first Rust-native jobs backend.
- Restate handler/workflow bridge behind `causlane-runtime/restate`; durable payload schema remains separate.
- Temporal/Dapr/Conductor experimental/community unless demand is clear.

## Policy adapters

- Cedar first for embedded fine-grained AuthZ.
- Casbin for simple RBAC/domain model.
- AuthZEN for external PDP interoperability.
- OpenFGA/SpiceDB for ReBAC as optional adapters.

## Observability adapters

- tracing connector first.
- JSON logs for local/dev.
- OpenTelemetry optional.
- Logging/metrics never become observed truth.

## Adapter certification suite

M08.7 codifies a bounded certification matrix for adapters that exist today.
Execution-bearing adapters pass by showing simulation to `GuardedExecutor`:

- hard effect cannot start before barrier;
- executor validates capability;
- adapter envelope/metadata cannot create semantic authority;
- authorized execution reaches the executor exactly once;
- produced refs survive the adapter wrapper.

Audit and observability adapters certify their own boundaries:

- append-only audit state rejects duplicate/non-monotonic truth writes;
- observability spans emit only after successful audit append;
- observability failure does not affect correctness.

`docs/product-track/adapter-certification-matrix.json` is the machine-readable
evidence ledger. Retry/idempotency for hard effects, cancellation/supersession
and durable truth-commit orchestration remain deferred to the reliability and
formal-depth milestones that make those semantics first-class.

## Shadow mode diagnostics

M08.8 adds a bounded shadow comparer for the feature-gated in-process runtime.
Host integrations subscribe to `InProcessRuntimeEvent`, supply
`ShadowExpectation` values, and call `compare_shadow_events` to receive a
`ShadowComparison`.

The comparer is diagnostic-only:

- it never admits, schedules, retries, cancels, blocks, or executes work;
- accepted events match by task lifecycle outcome, not ticket sequence;
- partition-scoped expectations can disambiguate duplicate task ids;
- rejected/failed expectations can match any error or an exact
  `HostDispatchError`;
- missing, mismatched, and unexpected observations are returned as data.

Full migration playbooks and reference integration rollout guidance remain in
the later M12.3 migration/shadow docs milestone.


---

<!-- 10-release-strategy.md -->

# 10. Release strategy

## Versioning

- `0.0.x`: experimental bootstrap releases. They may be private/internal or
  public pre-alpha, but they carry no API stability promise.
- `0.1.x`: first usable public alpha API surface.
- `0.5.x` or similar: beta once integrations validate API shape.
- `1.0.0`: stable API and compatibility policy.

The first crates.io publication target is `0.0.1`: a public pre-alpha bootstrap
release for package availability and provenance, not a stable alpha. It is still
blocked until the refactor-before-publication gate completes; local readiness
probes do not authorize upload by themselves.

## Crate publication strategy

The machine-derived dependency tiers and facade publish sequence are generated
in [`../release/publish-readiness.md`](../release/publish-readiness.md). Treat
that artifact as the source of truth for repository-local readiness and package
order; do not duplicate generated order by hand in product-track pages.

Actual crates.io upload is executed only through `PUBLISHING.md` and
[`../release/publish-all-crates-runbook.md`](../release/publish-all-crates-runbook.md).

Important distinction:

```text
publish-readiness report pass
  means deterministic repo-local no-upload checks passed.

actual upload readiness
  additionally requires clean history, secret scan, package-list inspection,
  staged registry dry-run and publication of internal dependencies in order.
```

## Stability policy

Stabilize first:

- core newtypes;
- AuditEvent/EventKind shapes;
- CompiledDispatchBundle shape;
- ReplayTrace shape;
- lifecycle stages;
- formal receipt schema.

Keep unstable first:

- runtime adapters;
- lane scheduler policy;
- proof generators beyond smoke/bounded checks;
- high-level macros/derive;
- UI/dashboard/service mode.

## Pre-alpha bootstrap criteria (`0.0.1`)

- refactor-before-publication gate records PUB0-PUB4 complete;
- package lists are manually reviewed;
- public docs say experimental/pre-alpha;
- AI/provenance policy exists;
- public GitHub baseline is curated;
- staged publish runbook is followed;
- crates do not overclaim workflow-engine, production-runtime or formal-proof
  readiness.

## Alpha criteria (`0.1.x`)

- runnable examples;
- docs/cookbook;
- replay corpus;
- public API review completed;
- feature flags/default features stabilized for alpha;
- no secret/context hygiene failures;
- clear non-goals.

## Beta criteria

- real integration feedback;
- adapter certification;
- performance baseline;
- migration/shadow mode;
- security docs.

## 1.0 criteria

- API freeze;
- semver/compat policy;
- formal/replay release gate;
- operational docs;
- at least one stable runtime adapter path;
- honest statement of proved/tested/bounded/out-of-scope properties.
