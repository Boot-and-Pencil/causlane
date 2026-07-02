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

`just formal-ready` и `just verification-full` проходят в чистой среде; coverage matrix выводится из receipts, а не из prose.

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
just verification-full
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
