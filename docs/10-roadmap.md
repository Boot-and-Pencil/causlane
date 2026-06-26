# Roadmap

This page is the short roadmap entrypoint. The detailed product-development
track lives in [`product-track/`](product-track/):

- [`product-track/00-executive-roadmap.md`](product-track/00-executive-roadmap.md)
  gives the executive view.
- [`product-track/01-product-track-map.md`](product-track/01-product-track-map.md)
  lists stages S00...S14 and exit gates.
- [`product-track/02-milestone-catalog.md`](product-track/02-milestone-catalog.md)
  is the milestone catalog.
- [`product-track/04-readiness-gates.md`](product-track/04-readiness-gates.md)
  defines quality, formal, replay, adapter, security, performance and docs gates.
- [`product-track/roadmap.json`](product-track/roadmap.json) and
  [`product-track/roadmap.yaml`](product-track/roadmap.yaml) are the
  machine-readable roadmap projections.

The current formal status remains governed by generated artifacts, receipts,
stale-check, coverage reports and formal-readiness status docs. The product
track is a planning corpus; it does not replace machine gates.

## Stage Mapping

| Legacy phase | Product-track stage |
|---|---|
| Phase 0 - Contract seed | S00 - Context, purpose and product frame |
| Phase 0.5 - Contract hardening | S01 - Contract foundation and formal readiness |
| Phase 1 - Formal model seed | S02 - Formal models v1 |
| Phase 2 - Reference kernel | S03 - Reference kernel and executable semantics |
| Phase 3 - Tooling | S04/S07 - Scenario/replay contract testing and DX |
| Phase 4 - Runtime adapters | S08 - Runtime shell and adapters |
| Phase 5 - Verification depth | S10 - Formal depth and proof hardening |
| Phase 6 - Public alpha | S11 - Public alpha and crate publication |

## Current Direction

The repository has already moved past the original contract-seed checklist:
compiled bundles, canonical hashes, Formal IR, generated formal artifacts,
receipts, stale-check, coverage derivation, replay controls and Lean4/Verus proof
facets now exist for the current slice.

The next product-level work should be read through the product-track stages:

1. Treat S01/S02/S03/S04 as advanced in-repo baselines backed by the current
   formal, replay, kernel and scenario/contract-testing gates.
2. S04 is closed in repo: golden scenarios (incl. conflict-free parallelism,
   projection and read-only sidecar via mixed-predicate traces), the full negative
   suite, one-source `scenario compile` (trace + formal IR + replay expectation),
   `replay verify --explain`, the `contract test` YAML harness and the
   mutation/fuzz harness all land and back the S04 exit gate.
3. S05 is now the active track: constraint plane / frontier scheduler; then
   S06/S07 add authz, explainability and supportability.
4. Use S08/S09/S10 to prove adapters, performance and deeper proof coverage
   before alpha/beta/1.0 release gates.

For the previous contract-hardening breakdown, see
[`11-contract-hardening-plan.md`](11-contract-hardening-plan.md).
