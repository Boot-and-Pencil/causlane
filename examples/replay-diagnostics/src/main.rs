#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_replay_diagnostics_example::run_replay_diagnostics() {
        Ok(summary) => {
            println!(
                "replay-diagnostics: explain accepted and rejected traces classified ({} accepted, {} rejected, {} located)",
                summary.accepted_cases, summary.rejected_cases, summary.located_rejections,
            );
        }
        Err(error) => {
            eprintln!("replay-diagnostics: {error}");
            std::process::exit(1);
        }
    }
}
