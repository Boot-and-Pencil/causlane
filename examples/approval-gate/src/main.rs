#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_approval_gate_example::run_approval_gate() {
        Ok(summary) => {
            println!(
                "approval-gate: approval satisfied; bundle replay verified ({} gate cases, {} scenario verified, {} scenario refuted)",
                summary.gate_cases, summary.verified_scenarios, summary.refuted_scenarios
            );
        }
        Err(error) => {
            eprintln!("approval-gate: {error}");
            std::process::exit(1);
        }
    }
}
