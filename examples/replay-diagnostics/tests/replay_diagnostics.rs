#![forbid(unsafe_code)]
#![deny(warnings)]

use causlane_replay_diagnostics_example::{collect_replay_diagnostics, run_replay_diagnostics};

#[test]
fn replay_diagnostics_summary_counts_acceptance_and_rejections(
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = run_replay_diagnostics()?;
    assert_eq!(summary.accepted_cases, 1);
    assert_eq!(summary.rejected_cases, 4);
    assert_eq!(summary.located_rejections, 3);
    assert_eq!(summary.json_reports, 5);
    assert_eq!(summary.human_reports, 5);
    Ok(())
}

#[test]
fn replay_diagnostics_classify_representative_failures() -> Result<(), Box<dyn std::error::Error>> {
    let reports = collect_replay_diagnostics()?;

    let execution = require_case(&reports, "execution_without_barrier")?;
    assert_eq!(execution.invariant.as_deref(), Some("I-001"));
    assert_eq!(
        execution.error_code.as_deref(),
        Some("ExecutionWithoutBarrier")
    );
    assert!(execution.has_location);

    let witness = require_case(&reports, "missing_witness")?;
    assert_eq!(witness.invariant.as_deref(), Some("I-009"));
    assert_eq!(
        witness.error_code.as_deref(),
        Some("RequiredWitnessMissing")
    );
    assert!(witness.has_location);

    let structural = require_case(&reports, "bundle_hash_mismatch")?;
    assert_eq!(structural.invariant, None);
    assert_eq!(structural.error_code.as_deref(), Some("BundleHashMismatch"));
    assert!(!structural.has_location);
    Ok(())
}

fn require_case<'a>(
    reports: &'a [causlane_replay_diagnostics_example::ReplayDiagnosticReport],
    case: &str,
) -> Result<&'a causlane_replay_diagnostics_example::ReplayDiagnosticReport, String> {
    reports
        .iter()
        .find(|report| report.case == case)
        .ok_or_else(|| format!("missing diagnostic report for {case}"))
}
