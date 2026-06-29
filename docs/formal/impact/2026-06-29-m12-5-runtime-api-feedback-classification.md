# Formal Impact Record: M12.5 runtime API feedback classification

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-runtime-api-feedback-classification
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F2 (developer/operator API validation evidence)

## Touched protocol-critical paths

```text
docs/product-track/api-validation-loop-plan.json
docs/product-track/
examples/runtime-operator-workflow/
tools/examples-check
```

## Summary

Classifies the M12.5 `runtime_dispatch_audit_projection` surface as
`accepted_for_freeze` for the current evidence slice. The classification consumes:

- synthetic examples:
  `runtime-guarded-audit-projection` and `runtime-operator-workflow`;
- property/fuzz evidence:
  runtime negative-control Rust tests and the `runtime_guarded_audit_projection`
  fuzz target;
- performance-scale evidence:
  `runtime_guarded_audit_projection_flow` from the dispatcher Criterion run.

The new `runtime-operator-workflow` example broadens synthetic coverage from a
single guarded operation to a multi-operation runtime host workflow. It uses
public `causlane` and `causlane-runtime` APIs only.

## API feedback classification

| Lane | Evidence | API feedback | Classification |
|---|---|---|---|
| Synthetic examples | multi-op guarded execution, audit trace projection and guarded dashboard projection redaction | Current public runtime composition APIs are usable without private helpers or API workarounds. | `accepted_for_freeze` |
| Property/fuzz | runtime negative controls and 15-minute dispatcher fuzz long-run with no crash or reproducer | No API shape change required from current findings. | `accepted_for_freeze` |
| Performance scale | 512 guarded ops, 1,028 audit events and one projection read measured at 1.4399 ms mean | No latency threshold or API shape change is introduced by this run. | `accepted_for_freeze` |

## Affected invariants

No invariant semantics change. The examples and classification exercise the
existing authorization, capability spend, audit append, trace projection and
projection redaction semantics without replacing their authorities.

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
| missing execution authorization | runtime example tests | deny before execution | pass |
| expired lease-derived capability | runtime example tests | capability spend refused | pass |
| missing projection authorization | runtime example tests | projection read denied | pass |
| wrong projection actor | runtime example tests | other actor's allow is not reusable | pass |
| duplicate audit event id | runtime example tests | duplicate rejected and no extra span emitted | pass |
| runtime fuzz long-run | dispatcher fuzz | no crash/reproducer artifacts | pass |
| runtime performance run | dispatcher Criterion | evidence recorded, no threshold claim | pass |

## Required proof/model changes

None.

## Not applicable lanes

This classification is limited to `runtime_dispatch_audit_projection`. Other
M12.5 selected surfaces remain pending until their own synthetic, fuzz,
performance and API feedback loops are classified.

## Acceptance commands

```bash
python3 tools/examples-check
./tools/api-validation-loop-plan-check
tools/product-track-bundle --check
tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: continue M12.5 classification for the remaining selected
  surfaces before M12.6 freeze planning consumes the full validation loop.
