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
