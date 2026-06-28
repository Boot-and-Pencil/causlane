#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_consequence_parallelism_example::run_consequence_parallelism() {
        Ok(summary) => {
            println!(
                "consequence-parallelism: frontier selected; replay verified ({} frontier cases, {} scenario verified, {} scenario refuted)",
                summary.frontier_cases, summary.verified_scenarios, summary.refuted_scenarios
            );
        }
        Err(error) => {
            eprintln!("consequence-parallelism: {error}");
            std::process::exit(1);
        }
    }
}
