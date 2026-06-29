# Formal Impact Record: M12.5 replay API feedback classification

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-replay-api-feedback-classification
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F2 (developer/operator API validation evidence)

## Touched protocol-critical paths

```text
docs/product-track/api-validation-loop-plan.json
docs/product-track/
examples/replay-operator-diagnostics/
tools/examples-check
```

## Summary

Classifies the M12.5 `replay_scenario_explain` surface as
`accepted_for_freeze` for the current evidence slice. The classification
consumes:

- synthetic examples:
  `replay-diagnostics`, `replay-operator-diagnostics` and existing replay
  examples that cover success, parallelism and why-not-parallel diagnostics;
- property/fuzz evidence:
  replay parse-boundary property tests, mutation-fuzz totality, the strict
  bundle-binding negative control and the `replay_trace_json` /
  `replay_scenario_yaml` fuzz targets;
- performance-scale evidence:
  `replay_verify_with_bundle` and `replay_explain_human` from the dispatcher
  Criterion run.

The new `replay-operator-diagnostics` example broadens synthetic coverage from
basic release-promotion explain output to an operator-facing workflow with
release execution, read-only sidecar projection, multi-action histories,
structural bundle provenance and strict bundle-binding controls. It uses public
`causlane-replay` and `causlane-contracts` APIs only.

## API feedback classification

| Lane | Evidence | API feedback | Classification |
|---|---|---|---|
| Synthetic examples | accepted traces, invariant-bearing rejections, structural bundle mismatch, human/JSON explain output and strict bundle binding | Current public replay APIs are usable without private helpers or API workarounds for the reviewed diagnostic workflow. | `accepted_for_freeze` |
| Property/fuzz | parse-boundary properties, mutation-fuzz totality, strict bundle negative control and two 15-minute dispatcher fuzz long-runs with no crash or reproducer | No API shape change required from current findings. | `accepted_for_freeze` |
| Performance scale | bundle-bound replay verification and human explain rendering measured on dispatcher | No latency threshold or API shape change is introduced by this run. | `accepted_for_freeze` |

## Affected invariants

No invariant semantics change. The examples and classification exercise the
existing replay verification, stable error-code, causal-location, human/JSON
diagnostic and strict bundle-binding semantics without replacing their
authorities.

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
| observed truth without execution | workflow example tests | `ObservedWithoutExecution` / `I-002` with action and plan location | pass |
| projection without anchor | workflow example tests | `ProjectionWithoutAnchor` / `I-003` with event location | pass |
| projection anchor wrong scope | workflow example tests | `AnchorAttestationMismatch` / `I-003` with event and anchor location | pass |
| event after lifecycle close | workflow example tests | `EventAfterClosed` / `I-008` with action and event location | pass |
| wrong trace bundle hash | workflow example tests | structural `BundleHashMismatch` and no causal location | pass |
| strict bundle hash missing | workflow example tests | `MissingTraceBundleHash` from strict replay | pass |
| replay fuzz long-runs | dispatcher fuzz | no crash/reproducer artifacts | pass |
| replay performance run | dispatcher Criterion | evidence recorded, no threshold claim | pass |

## Required proof/model changes

None.

## Not applicable lanes

This classification is limited to `replay_scenario_explain`.
`public_facade_and_core_kernel` remains pending until its own synthetic, fuzz,
performance and API feedback loop is classified.

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
  public facade/core surface before M12.6 freeze planning.
