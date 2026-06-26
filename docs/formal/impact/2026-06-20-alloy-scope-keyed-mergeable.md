# Formal Impact Record: scope-keyed Alloy mergeable + unrelated-merge control (P0-005 part 2)

## Change metadata

- Change ID: FIR-2026-06-20-alloy-scope-keyed-mergeable
- PR/issue: dispatcher-012 ТЗ, DoD fix stage (branch `hardening/found-problems`)
- Owner: repo maintainers
- Date: 2026-06-20
- Impact class: F3 (formal-model coverage / assertion correctness) — no kernel
  behavior change; replay semantics unchanged

## Touched protocol-critical paths

```text
crates/causlane-codegen/src/alloy.rs
crates/causlane-codegen/src/alloy_merge.rs        (new)
crates/causlane-codegen/src/lib.rs
crates/causlane-codegen/src/alloy_bindings_tests.rs
crates/causlane-cli/src/scenario_facts.rs
contracts/examples/release_promote_merge.registry.yaml                       (new)
contracts/scenarios/unrelated_merge_protocol_does_not_allow_conflict_invalid.scenario.yaml  (new)
formal/obligations/lifecycle_product_obligations.yaml
tools/formal-verify-all
```

## Summary

The generated Alloy conflict assertion was `some BundleFacts.mergeable or (no
exclusive conflicts)` with `mergeable: Predicate -> Predicate` pinned empty — so
ANY mergeable pair in the bundle would have globally disabled ALL exclusive-lease
conflict checks. It was only MVP-sound because `mergeable` was always empty.

This makes the relation scope-keyed and the relaxation per-scope, matching the
replay oracle:

- `mergeable: set LeaseScope`, pinned per scenario from the bundle by the shared
  `causlane_contracts::resolve_mergeable_scopes` (part 1), filtered to scopes that
  actually back a lease. `AlloyScenarioFacts` gains `predicate_id` + subject/
  circumstance bindings so the generator can resolve it; the new `alloy_merge`
  module holds the resolution (split for the 800-line cap).
- `GeneratedNoExclusiveConflicts` now relaxes a conflict ONLY when
  `a.scope in BundleFacts.mergeable`, never globally. `GeneratedMergeableDefaultEmpty`
  is emitted only for scenarios whose `mergeable` is empty.

Behavior is unchanged for every existing bundle (none declare an applicable verified
merge protocol, so `mergeable` resolves empty and conflicting_leases_invalid still
refutes). The new capability is proven by a fixture with a genuinely non-empty
`mergeable`.

## Non-vacuity proof (anti-theatre)

- New fixtures: `release_promote_merge.registry.yaml` makes the verified protocol
  `append_only_release_log_v1` APPLICABLE to `promote_release`, so its conflict
  domains (`environment:staging`, `release_candidate:rc_123`) are mergeable. The
  scenario `unrelated_merge_protocol_does_not_allow_conflict_invalid` holds one
  lease on the mergeable `environment:staging` plus two overlapping exclusive leases
  on the UNRELATED `queue:deploy`.
- On the merge bundle the generated `.als` pins
  `BundleFacts.mergeable = LeaseScope_environment_staging` (NON-empty), yet
  `GeneratedTraceSatisfiesCore` is **refuted** (status=fail) — the per-scope
  assertion catches the `queue:deploy` conflict. Verified the old global
  `some mergeable or …` would instead PASS (manually: injecting the conflict scope
  into `mergeable` relaxes it; an unrelated scope does not).
- Replay agrees: `ConflictingLeases { scope: "queue:deploy" }`.
- The gate wires `assert_alloy_refutes … "$MERGE_BUNDLE"` so the control runs on the
  applicable-merge bundle every run; the existing `conflicting_leases_invalid`
  control still refutes.

## Affected invariants

```text
I-006 (lease conflict / verified merge): the Alloy lane now models the verified-merge
relaxation per-scope (a 2nd lane for the merge nuance alongside replay). Kernel
semantics unchanged.
new invariant ids: none
```

## Affected formal models

```text
Alloy: GeneratedNoExclusiveConflicts is now scope-keyed (mergeable: set LeaseScope).
Negative control: unrelated_merge_protocol_does_not_allow_conflict_invalid (refuted
on the applicable-merge bundle). No P/Kani/Verus/Lean4 change.
```

## Contract changes

- New example registry + scenario (fixtures only).
- `AlloyScenarioFacts` gains `predicate_id`/`subject`/`circumstance` (codegen IR
  projection; not a bundle/trace field).
- Bundle / Formal IR data / replay-trace / receipt fields: none.
- Core semantic change: none.

## Required negative controls

| Scenario | Lane | Expected | Status |
|---|---|---|---|
| `unrelated_merge_protocol_does_not_allow_conflict_invalid` | Alloy (merge bundle) | `GeneratedTraceSatisfiesCore` refuted with `mergeable` NON-empty | new — verified pass→fail vs old assertion |
| `unrelated_merge_protocol_does_not_allow_conflict_invalid` | Replay | `ConflictingLeases` | new — refuted_by_replay |
| `conflicting_leases_invalid` | Alloy/Replay | still refute | existing — re-verified |
| `release_promote_success` | Alloy | pass (`mergeable = none`) | existing — re-verified |

## Required proof/model changes

| Lane | Artifact/check/theorem ID | Generated from IR? | Blocking profile |
|---|---|---:|---|
| Alloy | `GeneratedNoExclusiveConflicts` (scope-keyed) | generated | rust |
| Replay | `conflicting_leases_invalid`, `unrelated_merge_*` | yes | rust |
| P/Kani/Verus/Lean4 | unchanged | yes | rust/proof |

## Not applicable lanes

No P/Kani/Verus/Lean4 change; they already model the merge rule at the algebra level.

## Acceptance commands

```bash
just formal-verify-all
./tools/cargo-dev test -p causlane-codegen
```

## Exception request

- Exception needed? no
