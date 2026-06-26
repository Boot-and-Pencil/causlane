# Formal Impact Record: single claim-coverage predicate (dispatcher-012 P1-003)

## Change metadata

- Change ID: FIR-2026-06-19-claim-coverage-single-predicate
- PR/issue: dispatcher-012 ТЗ, DoD fix stage (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-19
- Impact class: F3 (enforcement-path consolidation) — behavior-preserving; no
  invariant semantics change

## Touched protocol-critical paths

```text
crates/causlane-core/src/domain/constraint.rs
crates/causlane-replay/src/lib.rs
```

## Summary

The "does a lease cover a resource claim" test was duplicated: core
`LeaseTable::validate_claim_coverage` and replay `validate_claim_manifest_coverage`
each inlined the same resource/scope/mode/amount match, risking divergence on what
"covered" means.

Extract the single predicate `causlane_core::lease_covers_claim(lease, claim)`
(same resource, scope and mode; lease amount ≥ claimed) and consult it from both:

- core `validate_claim_coverage` keeps its active-table lease set + holder
  (action/plan) binding, now matching via the shared predicate;
- replay `validate_claim_manifest_coverage` keeps its barrier-declared lease set +
  template scope resolution, now matching via the same predicate.

The distinct lease sets and binding rules at each call site are intentional
(different validation points) and unchanged; only the coverage-matching predicate
is unified. Behavior-preserving.

## Affected invariants

```text
I-006 (lease/claim coverage): the claim-coverage predicate is now single-sourced
across core and replay. Semantics unchanged; this removes a duplication divergence
risk, mirroring the I-006 lease-conflict single-authority consolidation.
new invariant ids: none
```

## Affected formal models

```text
none — no generated artifact change; no regeneration.
```

## Contract changes

- New public predicate `lease_covers_claim`.
- Bundle / Formal IR / trace / receipt / coverage fields: none.
- Core semantic change: none (behavior-preserving).

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `conflicting_leases_invalid` | Replay | `ConflictingLeases` | existing — re-verified |
| `release_promote_success` | Replay | pass (claims covered by barrier leases) | existing — re-verified |
| core `lease_table_rejects_conflicts_and_validates_claims` | unit | unchanged | existing — re-verified |

No new negative control: behavior is unchanged.

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Core/Replay | shared `lease_covers_claim` | n/a | rust |
| All formal lanes | unchanged | yes | rust |

## Not applicable lanes

No generated-model change; claim coverage is a replay/core runtime check.

## Acceptance commands

```bash
./tools/cargo-dev test -p causlane-core -p causlane-replay
just formal-verify-all
```

## Exception request

- Exception needed? no
