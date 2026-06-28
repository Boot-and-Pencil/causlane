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

Формальные модели v1 уже образуют рабочий verification contour (generated artifacts + receipts + stale-check + negative controls, `formal-verify-all` зелёный под default-профилем). Reference kernel (S03), сценарии/replay/contract testing (S04), constraint plane / frontier scheduler (S05), AuthZ/approval/capability plane (S06) и Observability/explainability/DX (S07) закреплены — `KernelContracts` единая runtime/replay authority; tiers/lanes, constraints/frontier, drain/fence, why-not-parallel, authz policy, approvals, scoped execution capabilities, projection redaction, tracing, support bundles and cookbook recipes кодифицированы чистыми модулями/docs с tests/replay coverage. Product engineering continues through S08/S09/S10; the current release-preparation track is S11/PUB5 staged crates.io publication after the public baseline and package file-list review.

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
| S07 | Observability, explainability и DX | advanced_in_repo | Сделать систему понятной и приятной: explain(), CLI, graph export, redaction, support bundles, docs/cookbook. |
| S08 | Runtime shell и adapters | active_next | Подключить execution backends и persistence без превращения Causlane в workflow engine. |
| S09 | Performance, reliability и high-throughput readiness | active_next | Сохранить correctness, но не превратить hot path в тяжелый workflow/control-plane overhead. |
| S10 | Formal depth и proof hardening | active_next | Углубить доказательства после стабилизации ядра: больше invariants, scopes, interleavings, preservation/refinement. |
| S11 | Public pre-alpha/bootstrap and alpha publication | active_pre_alpha_prep | Prepare public source/package provenance before public alpha. |
| S12 | Beta integrations и ecosystem validation | active_next | Проверить полезность на нескольких реальных интеграциях и улучшить DX/adapters. |
| S13 | 1.0 release readiness | planned | Стабилизировать публичный API, semver, invariants, docs, formal/replay gates и эксплуатационные гарантии. |
| S14 | Post-1.0: platform, ecosystem, advanced modes | future | Развивать control-plane/service mode, marketplace adapters, richer proofs, dashboards, distributed deployment. |


## Главные release criteria

### Pre-alpha bootstrap (`0.0.1`)

Pre-alpha bootstrap можно выпускать, когда:

- refactor-before-publication gate records PUB0-PUB4 complete;
- package file-list review is recorded;
- public docs say experimental/pre-alpha;
- GitHub public baseline is curated and scanned;
- staged publish runbook is followed;
- crates do not overclaim workflow-engine, production-runtime or formal-proof
  readiness.

### Alpha

Alpha можно выпускать, когда:

- core API компилируется и имеет минимально удобную facade;
- есть runnable examples;
- replay oracle покрывает основные valid/invalid сценарии;
- public API review completed;
- feature flags/default features stabilized for alpha;
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
