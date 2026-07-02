use vstd::prelude::*;

verus! {

enum Event {
    GateOpened,
    UseAttempted,
}

spec fn event_allowed(gate_seen: bool, event: Event) -> bool {
    match event {
        Event::GateOpened => true,
        Event::UseAttempted => gate_seen,
    }
}

spec fn step(gate_seen: bool, event: Event) -> bool {
    match event {
        Event::GateOpened => true,
        Event::UseAttempted => gate_seen,
    }
}

proof fn valid_gate_then_use_is_allowed()
    ensures
        event_allowed(false, Event::GateOpened),
        event_allowed(true, Event::UseAttempted),
        step(step(false, Event::GateOpened), Event::UseAttempted),
{
}

proof fn invalid_use_before_gate_is_rejected()
    ensures
        !event_allowed(false, Event::UseAttempted),
        !step(false, Event::UseAttempted),
{
}

}

fn main() {}
