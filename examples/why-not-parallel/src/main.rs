#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_why_not_parallel_example::run_why_not_parallel() {
        Ok(summary) => {
            println!(
                "why-not-parallel: blockers explained ({} cases, {} blocked, {} parallelizable)",
                summary.checked_cases, summary.blocked_cases, summary.parallelizable_cases
            );
        }
        Err(error) => {
            eprintln!("why-not-parallel: {error}");
            std::process::exit(1);
        }
    }
}
