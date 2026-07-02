// causlane core protocol model (generic, hand-written support model).
//
// Purpose (FM-001): ask "can a bad finite world exist under the kernel guards?"
// This model encodes the kernel enforcement guards (pred Enforced) and CHECKS
// that, under those guards, the dispatcher-critical safety invariants hold
// (I-001/I-002/I-003). The checks are run headlessly by
// verification/formal-full/tools/AlloyRunner.java with the bundled SAT4J solver.
//
// Bundle-specific facts (concrete predicates, scenarios) are GENERATED into
// verification/formal-full/alloy/generated/ from the compiled dispatch bundle (ADR-0014); this
// core model stays generic and reusable. ("hb" = happens-before; "before" is a
// reserved temporal keyword in Alloy 6.)
//
// AUTHORITY: this is a generic reusable checker; authoritative only with
// generated bundle/scenario facts. The hand-written core is NOT authority on
// its own — the compiled bundle + generated facts + receipts + stale-check are.
//
// Invariant coverage in THIS core: I-001, I-002, I-003 are checked directly.
// I-004 (overlay), I-005 (route/profile), I-006 (lease/merge), I-007 (drain),
// I-008 (replay acceptance), I-009 (approval binding) and I-010 (constraint
// update) are deferred for the Alloy lane and tracked in
// docs/formal-exceptions.md; I-006/I-008/I-009 are enforced today by the replay
// oracle and the generated negative controls, not by this generic core.

module core/causlane_core

abstract sig EventKind {}
one sig ActionAdmitted, ActionPlanned, DispatchLogged,
        ExecutionBarrierLogged, ExecutionStarted, ExecutionCompleted,
        ObservedTruthCommitted, ProjectionEmitted, LifecycleClosed,
        GateApproved, GateDenied, ConstraintLeaseGranted,
        ConstraintLeaseReleased, AuthzDecisionRecorded,
        ExecutionCapabilityIssued, DrainFenceRequested, DrainFenceAcquired,
        OverlayApplied, ConstraintUpdated, MergeProtocolVerified,
        ViolationDetected extends EventKind {}

sig Action {}
sig PlanHash {}

sig Event {
  action:   one Action,
  kind:     one EventKind,
  planHash: lone PlanHash,
  anchors:  set Event,
  hb:       set Event
}

// `hb` (happens-before) is a strict partial order: irreflexive and transitive.
fact StrictPartialOrder {
  no e: Event | e in e.hb
  all a, b, c: Event | (b in a.hb and c in b.hb) implies c in a.hb
}

// An anchor always references a strictly-earlier event.
fact AnchorsArePrior {
  all e: Event | e.anchors in e.hb
}

// Kernel enforcement guards — the rules causlane-core / causlane-replay impose.
pred Enforced {
  // I-001: execution only after a barrier for the same action + plan hash.
  all e: Event | e.kind = ExecutionStarted implies
    some b: Event |
      b.action = e.action and b.planHash = e.planHash and
      b.kind = ExecutionBarrierLogged and b in e.hb

  // I-002: observed truth only after execution for the same action + plan hash.
  all e: Event | e.kind = ObservedTruthCommitted implies
    some x: Event |
      x.action = e.action and x.planHash = e.planHash and
      x.kind = ExecutionStarted and x in e.hb

  // I-003: projection only with an anchor that is a prior observed truth.
  all e: Event | e.kind = ProjectionEmitted implies
    some a: e.anchors | a.kind = ObservedTruthCommitted

  // I-008: no event for an action may happen after that action's LifecycleClosed.
  all e: Event | e.kind = LifecycleClosed implies
    no late: Event | late.action = e.action and e in late.hb
}

// I-001 — under the guards, no execution lacks a prior barrier.
assert I_001_ExecutionRequiresBarrier {
  Enforced implies
    all e: Event | e.kind = ExecutionStarted implies
      some b: Event |
        b.action = e.action and b.kind = ExecutionBarrierLogged and b in e.hb
}

// I-002 — under the guards, no observed truth lacks a prior execution.
assert I_002_ObservedRequiresExecution {
  Enforced implies
    all e: Event | e.kind = ObservedTruthCommitted implies
      some x: Event |
        x.action = e.action and x.kind = ExecutionStarted and x in e.hb
}

// I-003 — under the guards, every projection has a prior observed-truth anchor.
assert I_003_ProjectionRequiresAnchor {
  Enforced implies
    all e: Event | e.kind = ProjectionEmitted implies
      some a: e.anchors | a.kind = ObservedTruthCommitted and a in e.hb
}

// I-008 — under the guards, no event happens after an action's LifecycleClosed.
assert I_008_NoEventAfterClosed {
  Enforced implies
    all e: Event | e.kind = LifecycleClosed implies
      no late: Event | late.action = e.action and e in late.hb
}

check I_001_ExecutionRequiresBarrier for 6
check I_002_ObservedRequiresExecution for 6
check I_003_ProjectionRequiresAnchor for 6
check I_008_NoEventAfterClosed for 6

// Consistency: a non-trivial valid trace (with a projection) exists under the
// guards — proves the guards are not contradictory/vacuous.
run ValidTraceExists {
  Enforced and some e: Event | e.kind = ProjectionEmitted
} for 6
