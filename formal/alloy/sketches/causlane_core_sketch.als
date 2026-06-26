// EXPLORATORY SKETCH — NOT AUTHORITATIVE.
// Do not extend into bundle-specific facts by hand. Real Alloy modeling is
// deferred until the contract is hardened and a generator exists; see
// ../../docs/11-contract-hardening-plan.md and ADR-0014. This file only
// explores notation and should eventually be generated from the compiled bundle.

abstract sig EventKind {}
one sig DispatchLogged, ExecutionBarrierLogged, ExecutionStarted, ObservedTruthCommitted, ProjectionEmitted extends EventKind {}

sig Action {}
sig Plan {}
sig Event {
  action: one Action,
  kind: one EventKind,
  before: set Event
}

sig Projection {
  action: one Action,
  anchor: lone Event
}

pred prior[e1, e2: Event] {
  e1 in e2.before
}

assert NoExecutionWithoutBarrier {
  all e: Event |
    e.kind = ExecutionStarted implies
      some b: Event |
        b.action = e.action and
        b.kind = ExecutionBarrierLogged and
        prior[b, e]
}

assert NoProjectionWithoutAnchor {
  all p: Projection |
    some p.anchor and p.anchor.kind = ObservedTruthCommitted
}

check NoExecutionWithoutBarrier for 5
check NoProjectionWithoutAnchor for 5
