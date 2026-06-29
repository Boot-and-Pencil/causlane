#![forbid(unsafe_code)]
#![deny(warnings)]

#[test]
fn facade_admission_accepts_action_call() -> Result<(), Box<dyn std::error::Error>> {
    causlane_facade_kernel_ergonomics_example::verify_facade_admission()?;
    Ok(())
}

#[test]
fn facade_barrier_policy_matches_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let checked = causlane_facade_kernel_ergonomics_example::verify_facade_barrier_policy()?;
    assert_eq!(checked, 2);
    Ok(())
}

#[test]
fn facade_frontier_reports_conflict_and_capacity() -> Result<(), Box<dyn std::error::Error>> {
    let summary = causlane_facade_kernel_ergonomics_example::verify_facade_frontier_selection()?;
    assert_eq!(summary.selected.len(), 2);
    assert_eq!(summary.write_scope_conflicts, 1);
    assert_eq!(summary.lane_capacity_rejections, 1);
    Ok(())
}

#[test]
fn facade_example_summary_counts_all_checks() -> Result<(), Box<dyn std::error::Error>> {
    let summary = causlane_facade_kernel_ergonomics_example::run_facade_kernel_ergonomics()?;
    assert_eq!(summary.accepted_admissions, 1);
    assert_eq!(summary.barrier_profiles_checked, 2);
    assert_eq!(summary.frontier_selected, 2);
    assert_eq!(summary.frontier_rejections, 2);
    Ok(())
}
