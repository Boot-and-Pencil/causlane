#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_release_orchestration_example::run_release_orchestration() {
        Ok(summary) => {
            println!(
                "release-orchestration: ci release graph verified ({} submitted, {} executed, {} audit events, {} packages reviewed, {} dry-run packages)",
                summary.submitted_tasks,
                summary.executed_tasks,
                summary.audit_events,
                summary.reviewed_packages,
                summary.dry_run_packages
            );
        }
        Err(error) => {
            eprintln!("release-orchestration: {error}");
            std::process::exit(1);
        }
    }
}
