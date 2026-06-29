#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_replay_operator_diagnostics_example::run_replay_operator_diagnostics() {
        Ok(summary) => {
            println!(
                "replay-operator-diagnostics: replay diagnostics workflow verified ({} accepted, {} rejected, {} controls)",
                summary.accepted_cases, summary.rejected_cases, summary.strict_negative_controls,
            );
        }
        Err(error) => {
            eprintln!("replay-operator-diagnostics: {error}");
            std::process::exit(1);
        }
    }
}
