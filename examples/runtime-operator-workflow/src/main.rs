#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_runtime_operator_workflow_example::run_runtime_operator_workflow() {
        Ok(summary) => {
            println!(
                "runtime-operator-workflow: guarded multi-op audit projection verified ({} executed, {} audit events, {} controls)",
                summary.executed_ops, summary.audit_events, summary.negative_controls,
            );
        }
        Err(error) => {
            eprintln!("runtime-operator-workflow: {error}");
            std::process::exit(1);
        }
    }
}
