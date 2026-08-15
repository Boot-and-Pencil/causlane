#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_reference_integration_example::run_reference_integration() {
        Ok(summary) => {
            println!(
                "reference-integration: api worker audit projection verified ({} submitted, {} executed, {} causal protocol events, {} projected fields, {} redacted)",
                summary.submitted_tasks,
                summary.executed_tasks,
                summary.causal_protocol_events,
                summary.projected_fields,
                summary.redacted_fields
            );
        }
        Err(error) => {
            eprintln!("reference-integration: {error}");
            std::process::exit(1);
        }
    }
}
