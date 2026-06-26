const P_MONITORS: &str = r#"// I-001, keyed by action (P1-001 interleaving lane): execution requires a
// prior barrier for the SAME action, so a barrier for one action cannot satisfy
// another's execution under interleaving. I-008 (no event after close) is owned by
// the keyed NoEventsAfterClosed below.
spec NoExecutionBeforeBarrier observes ExecutionBarrierLogged, ExecutionStarted {
  var barrierSeen: map[string, bool];
  start state Init {
    on ExecutionBarrierLogged do (p: EventPayload) { barrierSeen[p.actionId] = true; }
    on ExecutionStarted do (p: EventPayload) {
      assert p.actionId in barrierSeen, "I-001 execution requires a prior barrier for the same action";
    }
  }
}

// I-002, keyed by action: observed truth requires a prior execution for the SAME action.
spec NoObservedWithoutExecution observes ExecutionStarted, ObservedTruthCommitted {
  var executionSeen: map[string, bool];
  start state Init {
    on ExecutionStarted do (p: EventPayload) { executionSeen[p.actionId] = true; }
    on ObservedTruthCommitted do (p: EventPayload) {
      assert p.actionId in executionSeen, "I-002 observed truth requires execution for the same action";
    }
  }
}

// I-003, keyed by action: projection requires a prior observed truth for the SAME action.
spec NoProjectionWithoutAnchor observes ObservedTruthCommitted, ProjectionEmitted {
  var observedSeen: map[string, bool];
  start state Init {
    on ObservedTruthCommitted do (p: EventPayload) { observedSeen[p.actionId] = true; }
    on ProjectionEmitted do (p: EventPayload) {
      assert p.actionId in observedSeen, "I-003 projection requires prior observed truth for the same action";
    }
  }
}

// I-006, keyed by lease scope (P1-001 interleaving lane): two active exclusive
// leases conflict only when they hold the SAME scope, so concurrent exclusive
// leases on different scopes (e.g. environment:staging and release_candidate:rc_123
// admitted by one action) no longer false-conflict. Lease events are expanded one
// send per lease, so each lease's scope is observed individually.
spec NoConflictingActiveLeases observes ConstraintLeaseGranted, ConstraintLeaseReleased {
  var activeExclusive: map[string, bool];
  start state Init {
    on ConstraintLeaseGranted do (p: EventPayload) {
      if (p.leaseMode == "exclusive") {
        assert !(p.leaseScope in activeExclusive), "I-006 conflicting active exclusive leases on the same scope";
        activeExclusive[p.leaseScope] = true;
      }
    }
    on ConstraintLeaseReleased do (p: EventPayload) {
      if (p.leaseScope in activeExclusive) { activeExclusive -= (p.leaseScope); }
    }
  }
}

// I-007, keyed by scope: a drain fence over a scope blocks NEW mutable admissions
// on THAT scope only — a drain on one scope no longer blocks leases on another.
// Drain events carry the scope as factScope; lease grants carry it as leaseScope.
spec DrainBlocksNewMutableAdmission observes DrainFenceRequested, DrainFenceAcquired, ConstraintLeaseGranted {
  var draining: map[string, bool];
  start state Init {
    on DrainFenceRequested do (p: EventPayload) { draining[p.factScope] = true; }
    on ConstraintLeaseGranted do (p: EventPayload) {
      assert !(p.leaseScope in draining), "I-007 drain blocks new mutable admission on the same scope";
    }
    on DrainFenceAcquired do (p: EventPayload) {
      if (p.factScope in draining) { draining -= (p.factScope); }
    }
  }
}

// Planned I-012 evidence hook. Keyed by logical execution key
// (`exec:<planHash>:<opIndex>`): a hard execution for the same plan/op may not
// repeat even if a retry arrives as a different action.
spec NoDuplicateHardExecutionForSameIdempotencyKey observes ExecutionStarted {
  var executed: map[string, bool];
  start state Init {
    on ExecutionStarted do (p: EventPayload) {
      if (p.executionKey != "") {
        assert !(p.executionKey in executed), "duplicate hard execution for idempotency key";
        executed[p.executionKey] = true;
      }
    }
  }
}

// Planned I-014 evidence hook. Keyed by action: a recorded authz Deny
// before a barrier revokes any older Allow for that action; a later barrier or
// execution cannot ride stale authority.
spec AuthzRevocationBeforeBarrierBlocksExecution observes AuthzDecisionRecorded, ExecutionBarrierLogged, ExecutionStarted {
  var denied: map[string, bool];
  start state Init {
    on AuthzDecisionRecorded do (p: EventPayload) {
      if (p.authzDecision == "deny") { denied[p.actionId] = true; }
    }
    on ExecutionBarrierLogged do (p: EventPayload) { assert !(p.actionId in denied), "authz deny blocks barrier"; }
    on ExecutionStarted do (p: EventPayload) { assert !(p.actionId in denied), "authz deny blocks execution"; }
  }
}

// Planned I-018 evidence hook. Keyed by scope: lease-grant epochs must
// be nondecreasing for a scope, so a stale frontier/admission decision from an
// older constraint epoch cannot authorize a future grant after the scope moved on.
spec NoStaleConstraintEpochAdmission observes ConstraintLeaseGranted {
  var latestEpochByScope: map[string, int];
  start state Init {
    on ConstraintLeaseGranted do (p: EventPayload) {
      if (p.leaseScope != "") {
        if (p.leaseScope in latestEpochByScope) {
          assert p.leaseEpoch >= latestEpochByScope[p.leaseScope], "stale constraint epoch admission";
        }
        latestEpochByScope[p.leaseScope] = p.leaseEpoch;
      }
    }
  }
}

// I-008, keyed by action: once an action is closed, no further event for THAT action
// may occur — but other actions are unaffected (interleaving-correct).
spec NoEventsAfterClosed observes ExecutionStarted, ObservedTruthCommitted, ProjectionEmitted, ConstraintUpdated, LifecycleClosed {
  var closed: map[string, bool];
  start state Init {
    on LifecycleClosed do (p: EventPayload) { closed[p.actionId] = true; }
    on ExecutionStarted do (p: EventPayload) { assert !(p.actionId in closed), "I-008 event after closed"; }
    on ObservedTruthCommitted do (p: EventPayload) { assert !(p.actionId in closed), "I-008 event after closed"; }
    on ProjectionEmitted do (p: EventPayload) { assert !(p.actionId in closed), "I-008 event after closed"; }
    on ConstraintUpdated do (p: EventPayload) { assert !(p.actionId in closed), "I-008 event after closed"; }
  }
}

// I-009 coarse approval-before-barrier, keyed by action (the exact witness binding
// is WitnessFactGrounded, keyed by event).
spec ApprovalBindingDoesNotDrift observes GateApproved, ExecutionBarrierLogged {
  var approvalSeen: map[string, bool];
  start state Init {
    on GateApproved do (p: EventPayload) { approvalSeen[p.actionId] = true; }
    on ExecutionBarrierLogged do (p: EventPayload) { assert p.actionId in approvalSeen, "I-009 barrier requires prior approval witness for the same action"; }
  }
}

// I-010, keyed by action: a constraint update may not rewrite an action's committed
// truth, and an action's projection must derive from its committed truth.
spec ConstraintUpdateDoesNotRewriteTruth observes ObservedTruthCommitted, ProjectionEmitted, ConstraintUpdated {
  var truthCommitted: map[string, bool];
  start state Init {
    on ObservedTruthCommitted do (p: EventPayload) { truthCommitted[p.actionId] = true; }
    on ConstraintUpdated do (p: EventPayload) { assert !(p.actionId in truthCommitted), "I-010 constraint update cannot rewrite committed truth"; }
    on ProjectionEmitted do (p: EventPayload) { assert p.actionId in truthCommitted, "projection must derive from committed truth"; }
  }
}

// The full lifecycle chain, keyed by action: barrier -> execution -> truth ->
// projection must hold per action under interleaving.
spec ReplayAcceptsOnlyValidTrace observes ExecutionBarrierLogged, ExecutionStarted, ObservedTruthCommitted, ProjectionEmitted {
  var barrierSeen: map[string, bool];
  var executionSeen: map[string, bool];
  var truthSeen: map[string, bool];
  start state Init {
    on ExecutionBarrierLogged do (p: EventPayload) { barrierSeen[p.actionId] = true; }
    on ExecutionStarted do (p: EventPayload) { assert p.actionId in barrierSeen, "replay accepted execution without barrier"; executionSeen[p.actionId] = true; }
    on ObservedTruthCommitted do (p: EventPayload) { assert p.actionId in executionSeen, "replay accepted truth without execution"; truthSeen[p.actionId] = true; }
    on ProjectionEmitted do (p: EventPayload) { assert p.actionId in truthSeen, "replay accepted projection without truth"; }
  }
}

// Payload-bound (I-001): a capability must derive from a barrier seen for the
// SAME action+plan and covering its op. Discriminates a forged capability that
// references a non-barrier (or wrong-action/plan) event, per action/plan.
spec CapabilityBindsToBarrier observes ExecutionBarrierLogged, ExecutionStarted {
  var barrierAction: map[string, string];
  var barrierPlan: map[string, string];
  var barrierOp: map[string, int];
  start state Init {
    on ExecutionBarrierLogged do (p: EventPayload) {
      barrierAction[p.barrierId] = p.actionId;
      barrierPlan[p.barrierId] = p.planHash;
      barrierOp[p.barrierId] = p.opIndex;
    }
    on ExecutionStarted do (p: EventPayload) {
      assert p.barrierId in barrierAction, "I-001 capability references unknown barrier";
      assert barrierAction[p.barrierId] == p.actionId, "I-001 capability action mismatch";
      assert barrierPlan[p.barrierId] == p.planHash, "I-001 capability plan mismatch";
      assert barrierOp[p.barrierId] == p.opIndex, "I-001 capability op mismatch";
    }
  }
}

// Payload-bound (P0-004 producer attestation + I-009 exact binding): a witness
// ref's claimed (fact_kind, scope) must equal the attestation its producer event
// recorded about itself, and a bound gate witness must match the barrier's
// action, plan and impact set. Discriminates wrong-scope self-attestation and
// wrong action/plan/impact bindings — the same grounding replay and Alloy do.
spec WitnessFactGrounded observes GateApproved, ObservedTruthCommitted, ExecutionBarrierLogged {
  var attestedFact: map[string, string];
  var attestedScope: map[string, string];
  start state Init {
    on GateApproved do (p: EventPayload) {
      if (p.factKind != "" || p.factScope != "") {
        attestedFact[p.eventId] = p.factKind;
        attestedScope[p.eventId] = p.factScope;
      }
    }
    on ObservedTruthCommitted do (p: EventPayload) {
      if (p.factKind != "" || p.factScope != "") {
        attestedFact[p.eventId] = p.factKind;
        attestedScope[p.eventId] = p.factScope;
      }
    }
    on ExecutionBarrierLogged do (p: EventPayload) {
      if (p.claimEventId != "") {
        assert p.claimEventId in attestedFact, "P0-004 witness claims a fact from an event that attested none";
        assert attestedFact[p.claimEventId] == p.claimFactKind, "P0-004 witness claims a fact the producer did not attest";
        assert attestedScope[p.claimEventId] == p.claimScope, "P0-004 witness claims a scope the producer did not attest";
      }
      if (p.witnessBindAction != "" || p.witnessBindPlan != "" || p.witnessBindImpact != "") {
        assert p.witnessBindAction == p.actionId, "I-009 witness binds wrong action";
        assert p.witnessBindPlan == p.planHash, "I-009 witness binds wrong plan";
        assert p.witnessBindImpact == p.impactSetHash, "I-009 witness binds wrong impact";
      }
    }
  }
}

// Payload-bound (P0-004 producer attestation): a projection anchor's claimed
// (fact_kind, scope) must equal the attestation its observed-truth event
// recorded about itself. Discriminates a projection that claims a truth the
// observed event never recorded — the grounding replay/Alloy do.
spec AnchorFactGrounded observes ObservedTruthCommitted, ProjectionEmitted {
  var attestedFact: map[string, string];
  var attestedScope: map[string, string];
  start state Init {
    on ObservedTruthCommitted do (p: EventPayload) {
      if (p.factKind != "" || p.factScope != "") {
        attestedFact[p.eventId] = p.factKind;
        attestedScope[p.eventId] = p.factScope;
      }
    }
    on ProjectionEmitted do (p: EventPayload) {
      if (p.claimEventId != "") {
        assert p.claimEventId in attestedFact, "P0-004 projection anchor claims a fact from an event that attested none";
        assert attestedFact[p.claimEventId] == p.claimFactKind, "P0-004 projection anchor claims a fact the observed truth did not attest";
        assert attestedScope[p.claimEventId] == p.claimScope, "P0-004 projection anchor claims a scope the observed truth did not attest";
      }
    }
  }
}

// Payload-bound (P0-010): when a barrier references an authz decision, that
// decision must be an Allow. A recorded Deny (or a ref to a non-decision event)
// is refused — the same structural authz replay and Alloy enforce. Barriers with
// no authz ref (authz-disabled predicates) are unconstrained here, so this spec
// is safe on the main slice; missing-decision is caught by replay/Alloy.
spec AuthzDecisionGroundsBarrier observes AuthzDecisionRecorded, ExecutionBarrierLogged {
  var authz: map[string, string];
  start state Init {
    on AuthzDecisionRecorded do (p: EventPayload) {
      if (p.authzDecision != "") { authz[p.eventId] = p.authzDecision; }
    }
    on ExecutionBarrierLogged do (p: EventPayload) {
      if (p.authzRefEventId != "") {
        assert p.authzRefEventId in authz, "P0-010 barrier authz ref is not a recorded decision";
        assert authz[p.authzRefEventId] == "allow", "P0-010 barrier references a non-Allow authz decision";
      }
    }
  }
}
"#;

pub(crate) fn push_p_monitors(text: &mut String) {
    text.push_str(P_MONITORS);
}
