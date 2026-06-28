#![forbid(unsafe_code)]
#![deny(warnings)]

use causlane::core::protocol::{ActionId, FactKind, NotParallelReason, OpId, PairConflict, Scope};

fn op(action: &str, index: u32) -> OpId {
    OpId(ActionId(action.to_owned()), index)
}

#[test]
fn summary_matches_expected_counts() -> Result<(), Box<dyn std::error::Error>> {
    let summary = causlane_why_not_parallel_example::run_why_not_parallel()?;
    assert_eq!(summary.checked_cases, 4);
    assert_eq!(summary.blocked_cases, 3);
    assert_eq!(summary.parallelizable_cases, 2);
    Ok(())
}

#[test]
fn pair_conflict_case_reports_scope() -> Result<(), Box<dyn std::error::Error>> {
    let out = causlane_why_not_parallel_example::verify_pending_write_conflict()?;
    assert_eq!(
        out.conflict,
        PairConflict::WriteScopeConflict {
            scope: Scope("environment:staging".to_owned()),
        }
    );
    assert_eq!(
        out.rejected_reason,
        NotParallelReason::Frontier(
            causlane::core::protocol::FrontierBlock::WriteScopeConflict {
                scope: Scope("environment:staging".to_owned()),
                with: op("act_promote_123", 0),
            }
        )
    );
    Ok(())
}

#[test]
fn dependency_case_transitions_after_fact_production() -> Result<(), Box<dyn std::error::Error>> {
    let out = causlane_why_not_parallel_example::verify_dependency_blocker_transition()?;
    assert_eq!(
        out.before.reasons,
        vec![NotParallelReason::BlockedOnFact {
            fact: FactKind("approval:approved".to_owned()),
        }]
    );
    assert!(out.after.is_parallelizable());
    Ok(())
}

#[test]
fn active_writer_case_reports_holder() -> Result<(), Box<dyn std::error::Error>> {
    let out = causlane_why_not_parallel_example::verify_active_writer_blocker()?;
    assert_eq!(
        out.explanation.reasons,
        vec![NotParallelReason::BlockedOnActiveScope {
            scope: Scope("environment:staging".to_owned()),
            held_by: op("act_promote_active", 0),
        }]
    );
    Ok(())
}
