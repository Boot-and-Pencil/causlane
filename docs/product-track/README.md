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

## Implementation Handoff

- [`11-implementation-start-gate.md`](11-implementation-start-gate.md) is the
  checklist for starting the next milestone branch.
- [`12-milestone-execution-runbook.md`](12-milestone-execution-runbook.md)
  defines the milestone PR workflow, evidence expectations and negative-control
  discipline.
