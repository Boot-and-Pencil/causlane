# Invariant coverage matrix

The authoritative, machine-derived matrix is
[`coverage-matrix.json`](coverage-matrix.json), regenerated from the formal
coverage report (`target/causlane/formal-coverage-report.json`) by
`just formal-coverage-matrix`. `just verification-full` runs
`tools/coverage-matrix --check`, which **fails the gate** if this documentation
overclaims a lane the report does not back (P0-012). Do not hand-edit the lane
cells below; regenerate them.

Per-lane status (P0-006): a lane is `passed` only when a concrete, named check
(an Alloy assertion, a P spec, a Kani harness, a Lean4 theorem application, a refuted replay control) backs
the invariant on that lane; `pending_tool_run` means the generated check exists
but the current receipt has not recorded a passing run; `not_applicable` means
that lane does not model the invariant and another lane covers it. The
always-on Verus/Lean4 proof lanes read `passed` like the other lanes (they
gate every check-verification-full run). An invariant is `covered` when at least one lane proves it.

| ID | Invariant | replay | alloy | p | kani | verus | lean4 | Status |
|---|---|---|---|---|---|---|---|---|
| I-001 | Execution requires a prior write-ahead barrier. | passed | passed | passed | passed | passed | passed | covered |
| I-002 | Observed truth requires prior execution. | passed | passed | passed | passed | passed | passed | covered |
| I-003 | Projection requires an observed-truth anchor. | passed | passed | passed | passed | passed | passed | covered |
| I-004 | Overlay obligations may only strengthen the kernel contract. | not_applicable | not_applicable | not_applicable | passed | passed | passed | covered |
| I-005 | Routes and profiles cannot drift from the compiled bundle. | not_applicable | not_applicable | not_applicable | passed | passed | passed | covered |
| I-006 | Mutable lease conflicts require a verified merge protocol. | passed | passed | passed | passed | passed | passed | covered |
| I-007 | Drain fences require prior overlapping leases to clear. | passed | passed | passed | passed | passed | passed | covered |
| I-008 | No event may mutate lifecycle after terminal close. | passed | passed | passed | passed | passed | passed | covered |
| I-009 | Witness/authz evidence must bind exact action, plan and scope. | passed | passed | passed | passed | passed | passed | covered |
| I-010 | Constraint updates affect future frontier only. | not_applicable | not_applicable | passed | passed | passed | passed | covered |

## Derived lane summary

This section is generated from the same report as the table. It is kept in
Markdown for reader convenience, but `tools/coverage-matrix --check` compares
the whole file against the freshly generated body, so these summaries cannot
drift independently from the coverage report.

| Lane | Passed invariants | Not applicable | Pending tool run | Backing check_ids |
|---|---|---|---|---|
| replay | I-001, I-002, I-003, I-006, I-007, I-008, I-009 | I-004, I-005, I-010 | none | replay:execution_without_barrier_invalid, replay:observed_without_execution_invalid, replay:projection_without_anchor_invalid, replay:projection_anchor_wrong_fact_invalid, replay:projection_anchor_wrong_scope_invalid, replay:conflicting_leases_invalid, replay:drain_with_active_lease_invalid, replay:event_after_closed_invalid, replay:approval_wrong_plan_invalid, replay:approval_wrong_impact_invalid, replay:witness_wrong_scope_invalid |
| alloy | I-001, I-002, I-003, I-006, I-007, I-008, I-009 | I-004, I-005, I-010 | none | alloy:GeneratedTraceSatisfiesCore, alloy:GeneratedAnchorFactGrounded, alloy:GeneratedDrainFenceClear, alloy:GeneratedApprovalBindingHolds, alloy:GeneratedWitnessFactGrounded |
| p | I-001, I-002, I-003, I-006, I-007, I-008, I-009, I-010 | I-004, I-005 | none | p:NoExecutionBeforeBarrier, p:NoObservedWithoutExecution, p:NoProjectionWithoutAnchor, p:AnchorFactGrounded, p:NoConflictingActiveLeases, p:DrainBlocksNewMutableAdmission, p:NoEventsAfterClosed, p:WitnessFactGrounded, p:AuthzDecisionGroundsBarrier, p:ConstraintUpdateDoesNotRewriteTruth |
| kani | I-001, I-002, I-003, I-004, I-005, I-006, I-007, I-008, I-009, I-010 | none | none | kani:execution_requires_prior_barrier_nondet, kani:observed_truth_requires_prior_execution_nondet, kani:projection_anchor_source_kind_is_observed_truth_only, kani:overlay_never_weakens_obligations_nondet, kani:route_is_allowed_only_for_matching_profile_nondet, kani:lease_conflict_rule_is_fail_closed_without_verified_merge, kani:drain_fence_acquirable_only_without_active_overlap_nondet, kani:closed_stage_is_terminal_nondet, kani:witness_binding_is_exact_for_action_plan_and_impact, kani:constraint_update_cannot_rewrite_committed_truth_nondet |
| verus | I-001, I-002, I-003, I-004, I-005, I-006, I-007, I-008, I-009, I-010 | none | none | verus:execution_started_requires_prior_barrier, verus:observed_truth_requires_prior_execution, verus:projection_requires_prior_observed_truth, verus:overlay_accepted_never_weakens_obligations, verus:route_profile_compatibility, verus:lease_conflict_is_fail_closed_without_merge, verus:drain_after_overlap_clear, verus:closed_persists_across_step, verus:approval_binding_is_exact, verus:constraint_update_preserves_committed_truth |
| lean4 | I-001, I-002, I-003, I-004, I-005, I-006, I-007, I-008, I-009, I-010 | none | none | lean4:valid_trace_execution_started_has_prior_barrier, lean4:valid_trace_observed_truth_has_prior_execution, lean4:projection_anchor_soundness, lean4:overlay_monotonicity, lean4:route_profile_compatibility, lean4:lease_conflict_fail_closed, lean4:verified_merge_algebra, lean4:drain_after_overlap_clear, lean4:closed_is_terminal, lean4:witness_exact_binding, lean4:constraint_update_future_only |

## How to read this honestly

- The replay oracle is the executable authority for trace semantics. Each
  `passed` replay cell is grounded in a refuted invalid scenario or authz
  control, not a hand assertion.
- Alloy and P model structural/protocol relations where those lanes have
  generated facts or monitors. A `not_applicable` cell means the lane does
  not model that invariant; it is not hidden evidence.
- Kani, Verus and Lean4 cover only the cells backed by the generated
  `check_ids` above. The always-on proof lanes read `passed` only where their
  tool-run receipts carry the named obligation.

The full per-cell `check_ids` (which named check backs each `passed` lane) are in
[`coverage-matrix.json`](coverage-matrix.json) and the coverage report.

## Row template (for new invariants)

```text
ID:
Statement:
Authority surface:
Generated input:
Formal lanes (and the concrete check_id per lane):
Code-facing confirmation:
Runtime modules:
Readiness blocker:
Fresh receipt required:
Known gaps:
```
