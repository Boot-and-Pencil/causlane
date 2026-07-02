# causlane docs

This folder is the authoritative starting point for the project. Code should follow the normative documents, not the other way around.

## Recommended reading order

1. [`00-project-purpose.md`](00-project-purpose.md)
2. [`01-glossary.md`](01-glossary.md)
3. [`02-architecture-overview.md`](02-architecture-overview.md)
4. [`03-hexagonal-architecture.md`](03-hexagonal-architecture.md)
5. [`04-development-principles.md`](04-development-principles.md)
6. [`05-formal-modeling-strategy.md`](05-formal-modeling-strategy.md)
7. [`06-runtime-and-performance.md`](06-runtime-and-performance.md)
8. [`07-security-and-authz.md`](07-security-and-authz.md)
9. [`08-extension-process.md`](08-extension-process.md)
10. [`09-naming-and-publishing.md`](09-naming-and-publishing.md)
11. [`10-roadmap.md`](10-roadmap.md)
12. [`product-track/README.md`](product-track/README.md)
13. [`11-contract-hardening-plan.md`](11-contract-hardening-plan.md)
14. [`context-pack-hygiene.md`](context-pack-hygiene.md)

## ADRs

Architecture Decision Records live in [`adr/`](adr/). ADR-0001…0008 set the
foundations; ADR-0009…0015 pin the contract-hardening decisions (plan-hash
canonicalization, truth anchor vs witness, authz deny-by-default, merge-protocol
semantics, barrier witness/lease binding, the generate-don't-maintain pipeline,
and proposed formal evidence discipline).

## Product Track

The detailed product-development track lives in [`product-track/`](product-track/).
It is a planning corpus for stages, milestones and release gates; it does not
replace machine-derived formal status or gate outputs.

- [`product-track/00-executive-roadmap.md`](product-track/00-executive-roadmap.md)
- [`product-track/01-product-track-map.md`](product-track/01-product-track-map.md)
- [`product-track/02-milestone-catalog.md`](product-track/02-milestone-catalog.md)
- [`product-track/04-readiness-gates.md`](product-track/04-readiness-gates.md)
- [`product-track/10-release-strategy.md`](product-track/10-release-strategy.md)
- [`product-track/roadmap.json`](product-track/roadmap.json)
- [`product-track/roadmap.yaml`](product-track/roadmap.yaml)

## Release And Publication

The current public-release handoff lives under [`release/`](release/). Start
with [`release/publication-prep.md`](release/publication-prep.md), then use
[`../PUBLISHING.md`](../PUBLISHING.md) and
[`release/publish-all-crates-runbook.md`](release/publish-all-crates-runbook.md)
for the staged crates.io sequence. Generated readiness status remains
[`release/publish-readiness.md`](release/publish-readiness.md); do not hand-edit
it.

## Formal Lifecycle

The current formal gate authority remains generated artifacts, receipts,
stale-check and derived coverage. The proposed full lifecycle discipline lives
under [`formal/`](formal/):

- [`formal/00-product-lifecycle-formal-map.md`](formal/00-product-lifecycle-formal-map.md)
- [`formal/01-formal-model-catalog.md`](formal/01-formal-model-catalog.md)
- [`formal/02-protocol-catalog.md`](formal/02-protocol-catalog.md)
- [`formal/03-lean4-verus-proof-obligations.md`](formal/03-lean4-verus-proof-obligations.md)
- [`formal/04-formal-discipline-and-anti-theatre.md`](formal/04-formal-discipline-and-anti-theatre.md)
- [`formal/05-feature-fix-gating.md`](formal/05-feature-fix-gating.md)
- [`formal/06-model-code-alignment.md`](formal/06-model-code-alignment.md)
- [`formal/07-integration-tz.md`](formal/07-integration-tz.md)

Machine-readable catalog/seed files are
[`formal/formal_model_catalog.yaml`](formal/formal_model_catalog.yaml) and
[`../verification/formal-full/obligations/lifecycle_product_obligations.yaml`](../verification/formal-full/obligations/lifecycle_product_obligations.yaml).
The discipline checker contract is documented in
[`../tools/specs/formal-discipline-check.md`](../tools/specs/formal-discipline-check.md).

## Templates

Formal lifecycle templates live in [`templates/`](templates/):

- [`formal-impact-record.md`](templates/formal-impact-record.md)
- [`formal-obligation-record.md`](templates/formal-obligation-record.md)

## Scenarios

Narrative scenarios live in [`scenarios/`](scenarios/). Executable scenario
fixtures live in [`../contracts/scenarios/`](../contracts/scenarios/) and emit
replay traces with `causlane scenario emit-trace`. The same YAML can bind
generated formal facts through `formal generate alloy --scenario ...`.

Start with the [`scenarios/cookbook.md`](scenarios/cookbook.md) recipes for the
current executable CLI surfaces: bundle, scenario, replay, graph, support bundle
and adapter-boundary checks.

## Invariants

Dispatcher-critical invariants are tracked in [`invariants/coverage-matrix.md`](invariants/coverage-matrix.md).

## Host integration

- [Host Dispatch API v2](specs/host-dispatch-api-v2.md) defines the current `causlane.host-dispatch.v2` seam with partition routing.
- [Host Dispatch API v1](specs/host-dispatch-api-v1.md) remains the historical linear seam.
- [ADR-0017](adr/0017-host-dispatch-api-v2-partition-coordinator.md) records the v2 partition coordinator decision.
