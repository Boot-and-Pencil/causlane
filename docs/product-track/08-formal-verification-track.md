# 08. Formal verification track

## Roles

```text
Alloy  -> relational bad worlds.
P      -> message/order/interleaving bad histories.
Kani   -> bounded checks of Rust reducers/validators.
Verus  -> abstract preservation/refinement proofs.
Lean4  -> optional generated theorem applications / proof facet.
Replay -> executable oracle over real traces.
```

## v1 invariant scope

Blocking first set:

- I-001 no execution without barrier;
- I-002 no observed truth without execution;
- I-003 no projection without observed-truth anchor;
- I-006 no conflicting mutable frontier without merge protocol;
- I-008 replay accepts only valid protocol traces;
- I-009 approval/witness exact binding.

Second set:

- I-004 overlay cannot weaken obligations;
- I-005 route derives from consequence profile;
- I-007 drain acquired only after prior overlapping leases clear;
- I-010 constraint updates affect future frontier, not past truth.

## Alloy v1

Inputs:

- Formal IR generated from bundle/scenario;
- generic core model;
- generated facts and checks;
- negative controls.

Assertions:

- NoExecutionWithoutBarrier;
- NoObservedWithoutExecution;
- NoProjectionWithoutAnchor;
- NoConflictingMutableFrontier;
- WitnessFactGrounded;
- AnchorFactGrounded;
- ApprovalBoundToPlanImpact;
- RouteHasConsequenceProfile.

## P v1

Machines/monitors:

- Dispatcher, AuditLog, LeaseManager, Worker, ConstraintProvider, ProjectionBuilder;
- NoExecutionBeforeBarrier;
- NoProjectionBeforeObservedAnchor;
- DrainBlocksNewMutableAdmission;
- NoDuplicateHardExecutionOnRetry;
- AuthzAllowBeforeBarrier.

## Kani v1

Harnesses:

- reduce_lifecycle;
- Replay accepts => trace protocol properties;
- LeaseTable no overlapping exclusive writes;
- ExecutionCapability derives only from valid barrier;
- parser/decoder fail-closed/no panic;
- quota/capacity no underflow/overflow.

## Verus v1

Proofs:

- lifecycle preservation;
- barrier-before-execution theorem;
- projection-anchor theorem;
- overlay monotonicity;
- lease map preservation;
- replay_accepts => valid_trace for abstract model.

## Lean4 v1

Keep generated and narrow. Lean4 is now release-blocking inside
`formal-verify-all`; broader theorem applications must still be grounded in
generated artifacts, receipts and coverage rows before they are claimed.

## Formal anti-theatre rules

- No lane can claim coverage without concrete check_id and fresh receipt.
- Non-blocking proof facet cannot roll up to covered unless release profile says so.
- Negative controls must fail for expected reason.
- Exceptions must have expiry and owner.
