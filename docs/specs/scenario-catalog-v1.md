# Scenario catalog v1 (FM-006)

The scenario catalog (`contracts/scenarios/*.scenario.yaml`) is the
invariant-indexed corpus that drives replay and the generated formal artifacts.
Each scenario is validated by `contracts/schema/scenario.schema.json` and lowered
to a trace by `ReplayScenario::to_trace()`.

## Required fields

```yaml
scenario_version: 0.1.0
scenario_id: release_promote_success      # stable id; *_invalid.* => negative control
predicate: release.promote_candidate
action_id: act_promote_123
plan_hash: sha256:...
expected_replay_result: pass | fail        # the executable-oracle outcome
expected_error_code: <ReplayErrorCode>     # required for negative scenarios
formal_obligations: [I-001, I-002, ...]    # invariants this scenario exercises
# FM-006 invariant / target indexing (optional, additive):
invariants:
  positive: [I-001, I-002, I-003, I-006, I-008, I-009]
  negative: []
targets:
  alloy: required | optional | abstract_only | not_applicable
  p: required | optional | not_applicable
  kani: optional | not_applicable
  verus: abstract_only | not_applicable
  replay: required
events: [ ... ordered trace events ... ]
```

Notes:

- `expected_replay_result` is the field consumed by the Rust oracle today.
  `invariants`/`targets` are additive indexing blocks (the YAML parser ignores
  unknown keys); they document which lanes a scenario is meant to feed.
- Negative scenarios (`*_invalid.scenario.yaml`) MUST declare
  `expected_error_code`. `causlane-formal verify-all` runs **every** negative
  scenario through `verify_with_bundle` and fails the gate (`BROKEN: …`) if any
  is not refuted with its exact declared code (P-003).

## Catalog status

Real, replay-refuted negative controls (each verified to fail with the declared
`ReplayErrorCode`):

| scenario | expected code | invariant |
|---|---|---|
| `execution_without_barrier_invalid` | `ExecutionWithoutBarrier` | I-001 |
| `execution_without_capability_invalid` | `CapabilityMissing` | I-001 |
| `forged_capability_invalid` | `CapabilityMismatch` | I-001 |
| `observed_without_execution_invalid` | `ObservedWithoutExecution` | I-002 |
| `projection_without_anchor_invalid` | `ProjectionWithoutAnchor` | I-003 |
| `projection_anchor_wrong_plan_invalid` | `AnchorNotObservedTruth` | I-003 |
| `projection_anchor_wrong_fact_invalid` | `AnchorAttestationMismatch` | I-003 / I-009 |
| `projection_anchor_wrong_scope_invalid` | `AnchorAttestationMismatch` | I-003 / I-009 |
| `conflicting_leases_invalid` | `ConflictingLeases` | I-006 |
| `drain_with_active_lease_invalid` | `DrainFenceWithActiveOverlap` | I-007 |
| `event_after_closed_invalid` | `EventAfterClosed` | I-008 / I-010 |
| `approval_wrong_plan_invalid` | `WitnessBindingMismatch` | I-009 |
| `approval_wrong_impact_invalid` | `WitnessBindingMismatch` | I-009 |
| `missing_witness_invalid` | `RequiredWitnessMissing` | I-009 |
| `witness_wrong_scope_invalid` | `WitnessSelectorMismatch` | I-009 |
| `witness_event_wrong_fact_kind_invalid` | `WitnessAttestationMismatch` | I-009 |

Not yet expressible (need trace-schema extensions — tracked in
`docs/formal-exceptions.md`): overlay (I-004) negatives and the provenance
negatives (`stale_generated_artifact_invalid`,
`stale_receipt_bundle_hash_invalid`), which are exercised by
`formal stale-check` rather than replay.

## Adding a predicate without breaking coverage

1. Add the predicate to the registry; recompile the bundle.
2. Add a `*_success.scenario.yaml` exercising the full lifecycle.
3. For each P0 invariant the predicate touches, add a `*_invalid.scenario.yaml`
   with the exact `expected_error_code`.
4. Run `just formal-verify-all` — new negatives must be `refuted_by_replay`.
