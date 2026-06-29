# Formal Impact Record: M12.5 contracts API feedback classification

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-contracts-api-feedback-classification
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F2 (developer/operator API validation evidence)

## Touched protocol-critical paths

```text
docs/product-track/api-validation-loop-plan.json
docs/product-track/
examples/contracts-registry-bundle-workflow/
tools/examples-check
```

## Summary

Classifies the M12.5 `contracts_registry_bundle_plan_hash` surface as
`accepted_for_freeze` for the current evidence slice. The classification
consumes:

- synthetic examples:
  `contracts-boundary-ergonomics` and `contracts-registry-bundle-workflow`;
- property/fuzz evidence:
  existing registry parse/compile property tests, the new workflow negative
  controls and the `registry_yaml_compile` fuzz target;
- performance-scale evidence:
  `registry_normalize_from_yaml`, `plan_hash_release_promote` and
  `bundle_load_from_json` from the dispatcher Criterion run.

The new `contracts-registry-bundle-workflow` example broadens synthetic coverage
from boundary ergonomics to a near-real multi-predicate registry workflow. It
uses public `causlane-contracts` APIs only.

## API feedback classification

| Lane | Evidence | API feedback | Classification |
|---|---|---|---|
| Synthetic examples | multi-predicate registry manifest, bundle validation/compilation, artifact reload, template resolution, plan-template cache, plan hash and impact hash | Current public contracts APIs are usable without private helpers or API workarounds for the reviewed workflow. | `accepted_for_freeze` |
| Property/fuzz | registry parse/compile property tests, workflow negative controls and 15-minute dispatcher fuzz long-run with no crash or reproducer | No API shape change required from current findings. | `accepted_for_freeze` |
| Performance scale | registry normalization, plan-hash computation and bundle JSON load measured on dispatcher | No latency threshold or API shape change is introduced by this run. | `accepted_for_freeze` |

## Affected invariants

No invariant semantics change. The examples and classification exercise the
existing registry validation, bundle compilation, authz policy checks, template
resolution and plan-hash semantics without replacing their authorities.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public Rust API: none.
- Manifest versions: none.
- Production dependencies: none.
- Generated publication readiness reports: none.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| tampered compiled bundle artifact | workflow example tests | deserialization rejects missing predicate data | pass |
| missing template binding | workflow example tests | template resolver reports missing binding | pass |
| blank required authz policy id | workflow example tests | bundle validator rejects the registry | pass |
| plan material mutation | workflow example tests | plan hash changes | pass |
| registry YAML compile fuzz long-run | dispatcher fuzz | no crash/reproducer artifacts | pass |
| contracts performance run | dispatcher Criterion | evidence recorded, no threshold claim | pass |

## Required proof/model changes

None.

## Not applicable lanes

This classification is limited to `contracts_registry_bundle_plan_hash`.
`public_facade_and_core_kernel` and `replay_scenario_explain` remain pending
until their own synthetic, fuzz, performance and API feedback loops are
classified.

## Acceptance commands

```bash
python3 tools/examples-check
./tools/api-validation-loop-plan-check
tools/product-track-bundle --check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: continue M12.5 classification for the remaining pending
  public facade/core and replay/explain surfaces before M12.6 freeze planning.
