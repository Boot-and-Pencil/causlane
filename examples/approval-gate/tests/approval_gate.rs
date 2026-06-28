#![forbid(unsafe_code)]
#![deny(warnings)]

#[test]
fn local_approval_gate_cases_match_expected_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let checked = causlane_approval_gate_example::verify_local_approval_gate_cases()?;
    assert_eq!(checked, 7);
    Ok(())
}

#[test]
fn release_promote_scenario_replays_with_bound_approval() -> Result<(), Box<dyn std::error::Error>>
{
    causlane_approval_gate_example::verify_release_promote_success()?;
    Ok(())
}

#[test]
fn wrong_plan_approval_witness_is_refuted() -> Result<(), Box<dyn std::error::Error>> {
    causlane_approval_gate_example::refute_wrong_plan_approval()?;
    Ok(())
}
