# Formal Impact Record: M12.5 runtime guarded fuzz/property slice

## Change metadata

- Change ID: FIR-2026-06-29-m12-5-runtime-guarded-fuzz-property
- PR/issue: M12.5 API validation loop
- Owner: repo maintainers
- Date: 2026-06-29
- Impact class: F1 (test/tooling only)

## Touched protocol-critical paths

```text
fuzz/
docs/product-track/api-validation-loop-plan.json
docs/product-track/milestones/m12.5-api-validation-loop.md
```

## Summary

Adds the first runtime authz/audit/projection fuzz target for the M12.5 API
validation loop. The target byte-drives the public `causlane` facade and
`causlane-runtime` APIs for:

- guarded execution authorization and capability spend;
- in-memory audit append plus trace projection;
- guarded projection authorization and redaction partitioning.

The target does not re-implement authorization, capability, audit or redaction
semantics. It checks the outcomes returned by the existing public runtime and
kernel authorities.

## Affected invariants

No invariant semantics change. The slice strengthens property/fuzz evidence for
the already implemented fail-closed runtime execution, audit append and
projection-read behavior.

## Affected formal models

None. No generated model, receipt, coverage matrix, Formal IR, Alloy, P, Kani,
Verus or Lean artifact changes.

## Contract changes

- Bundle / replay trace / scenario / receipt schemas: none.
- Public API: none.
- Production dependencies: none.
- Dev/test dependencies: `causlane` and `causlane-runtime` for the detached fuzz
  crate only.

## Required negative controls

| Scenario | Expected lane | Expected check | Status |
|---|---|---|---|
| invalid execution authz, expired lease or uncovered op | cargo-fuzz | guarded execution refuses before returning executor marker | new |
| duplicate audit event id | cargo-fuzz | audit append fails and trace span count does not grow | new |
| invalid `may_project` decision | cargo-fuzz | projection read is denied | new |
| projection redaction mask | cargo-fuzz | revealed/redacted sets partition requested fields by allowlist | new |

## Required proof/model changes

None.

## Not applicable lanes

Generated formal lanes are unaffected. Long-running fuzz execution remains part
of the M12.5 evidence collection before terminal API classification.

## Acceptance commands

```bash
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 test --manifest-path fuzz/Cargo.toml --no-run --bins --locked
REAL_CARGO="$(command -v cargo)" DEVINFRA_ALLOW_DIRECT_CARGO=1 ./tools/cargo +nightly-2025-11-21 fuzz run runtime_guarded_audit_projection -- -runs=1 -artifact_prefix=/tmp/causlane-fuzz-artifacts/
./tools/api-validation-loop-plan-check
./tools/formal-discipline-check --profile base --from-git origin/main...HEAD
```

## Exception request

- Exception needed? no
- Follow-up issue: run a longer `runtime_guarded_audit_projection` fuzz profile
  on the dispatcher host and record any reproducer as curated corpus plus an API
  feedback finding.
