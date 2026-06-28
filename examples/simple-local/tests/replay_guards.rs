#![forbid(unsafe_code)]
#![deny(warnings)]

use causlane::prelude::AuditEventKind;
use causlane_replay::ReplayError;

#[test]
fn simple_local_replay_verifies() -> Result<(), Box<dyn std::error::Error>> {
    let events = causlane_simple_local_example::simple_local_events()?;
    causlane_replay::verify_events(&events)?;
    Ok(())
}

#[test]
fn missing_barrier_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let mut events = causlane_simple_local_example::simple_local_events()?;
    events.retain(|event| event.kind != AuditEventKind::ExecutionBarrierLogged);

    let result = causlane_replay::verify_events(&events);
    assert!(matches!(
        result,
        Err(ReplayError::ExecutionWithoutBarrier { .. })
    ));
    Ok(())
}
