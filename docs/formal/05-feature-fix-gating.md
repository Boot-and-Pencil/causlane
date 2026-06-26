# Feature and fix gating policy

> **Repository integration status:** proposed lifecycle discipline. This
> document is design/process authority only; current proof evidence remains the
> generated chain from compiled bundle and scenario through Formal IR, generated
> artifacts, receipts, stale-check and derived coverage.

> **Implementation note for repo 010:** `tools/formal-discipline-check` is
> implemented for local and PR-diff checks and is mandatory inside
> `tools/formal-verify-all` after fresh coverage and coverage-matrix drift
> checks. Provider-specific CI workflow adoption is outside this repo.

## Rule

For protocol-critical changes, **formal obligation comes before implementation**.

This does not require finishing every proof before writing any code, but it does require naming the obligation, adding the negative control or proof target, and wiring the gate so the gap is visible.

## Before coding a feature

Create or update `docs/templates/formal-impact-record.md` and include it in the PR description or a tracked `docs/formal/impact/*.md` file.

Required fields:

```text
change id
owner
touched protocol-critical paths
affected invariant ids
affected protocol ids
affected model ids
new/changed bundle fields
new/changed Formal IR fields
new/changed replay errors
new/changed generated artifacts
required negative controls
required proof obligations
lane status before code
acceptance command
exception request, if any
```

## Before coding a fix

If the fix changes safety behavior:

1. add or identify a scenario that fails with the current bug;
2. record the expected stable error code or monitor failure;
3. ensure the scenario is included in the negative-control catalog;
4. only then implement the fix;
5. prove the fix by the gate.

If no scenario can reproduce the bug, explain why and add a different discriminating control: Kani harness, P interleaving, Lean/Verus theorem, or tool-level stale/coverage check.

## Formal impact classification

| Class | Meaning | Required action |
|---|---|---|
| F0 | docs-only, no protocol effect | no formal gate beyond docs checks |
| F1 | tests/scenarios only | run replay/scenario gate |
| F2 | bundle/IR/schema change | update codegen, stale-check, receipts, coverage |
| F3 | kernel invariant change | update model catalog, negative controls, Kani/Verus/Lean obligations |
| F4 | runtime hard-effect path | require capability/barrier/lease/authz replay controls and guarded executor checks |
| F5 | formal tooling/reporting | add anti-overclaim tests and receipt/coverage non-upgrade checks |

## Mandatory mapping

Every F2+ change must update at least one of:

```text
formal/obligations/lifecycle_product_obligations.yaml
docs/formal/formal_model_catalog.yaml
docs/invariants/coverage-matrix.json via generated report
docs/formal/proof-refinement-scope.json when claim strength changes
docs/formal-exceptions.json when a temporary non-blocking exception is needed
```

## CI policy

Protocol-critical PRs currently must pass the implemented mandatory gates:

```bash
just formal-ready
just formal-verify-all
```

Available PR-diff discipline check for provider CI or local review:

```bash
tools/formal-discipline-check --profile rust --changed-files <files>
```

This check is strict about formal evidence mapping: every manifest-required
`check_id` must be present in coverage/docs and backed by generated artifact
text or replay negative controls plus codegen/tool-run receipt obligations.

For proof-lane PRs:

```bash
tools/full-doctor --json --profile proof
just formal-verify-all --profile proof
```

## Merge-blocking examples

```text
new AuditEvent kind without replay grammar and Formal IR projection
new route profile without route/profile model update
new authz policy field without replay binding and scenario controls
new merge protocol marked verified without Lean/Verus/Kani obligations
new runtime executor path that spends raw leases instead of capability
new coverage table row without artifact-present check_id
new Verus theorem that proves a frozen flag unrelated to transition
```

## Emergency fixes

Emergency production fixes may merge behind an explicit exception only if:

1. exception is in executable JSON policy;
2. expiry is short and concrete;
3. the skipped lane is not forbidden by `formal/proof-lanes.json`;
4. a follow-up issue names the missing model/control;
5. the gate records `non_blocking_skipped`, never `passed`.
