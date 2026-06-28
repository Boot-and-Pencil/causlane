#![forbid(unsafe_code)]
#![deny(warnings)]

#[test]
fn local_frontier_cases_match_expected_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let checked = causlane_consequence_parallelism_example::verify_frontier_cases()?;
    assert_eq!(checked, 3);
    Ok(())
}

#[test]
fn conflict_free_parallelism_scenario_replays() -> Result<(), Box<dyn std::error::Error>> {
    causlane_consequence_parallelism_example::verify_conflict_free_parallelism_replay()?;
    Ok(())
}

#[test]
fn overlapping_parallelism_is_refuted() -> Result<(), Box<dyn std::error::Error>> {
    causlane_consequence_parallelism_example::refute_overlapping_parallelism()?;
    Ok(())
}
