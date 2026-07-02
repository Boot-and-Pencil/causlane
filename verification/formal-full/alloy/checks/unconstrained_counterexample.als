// Negative control (expected_fail).
//
// WITHOUT the kernel guards, a bad world exists: an ExecutionStarted with no
// prior ExecutionBarrierLogged. The Alloy runner MUST report this check as NOT
// holding (satisfiable counterexample). This proves the lane discriminates and
// is not passing vacuously. Expected runner result: status "fail", exit 1.

abstract sig EventKind {}
one sig ExecutionStarted, ExecutionBarrierLogged extends EventKind {}

sig Action {}
sig Event {
  action: one Action,
  kind:   one EventKind,
  hb:     set Event
}

fact Irreflexive { no e: Event | e in e.hb }

assert NoExecutionWithoutBarrier {
  all e: Event | e.kind = ExecutionStarted implies
    some b: Event |
      b.action = e.action and b.kind = ExecutionBarrierLogged and b in e.hb
}

check NoExecutionWithoutBarrier for 4
