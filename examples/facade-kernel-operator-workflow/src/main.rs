#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_facade_kernel_operator_workflow_example::run_facade_kernel_operator_workflow() {
        Ok(summary) => {
            println!(
                "facade-kernel-operator-workflow: facade kernel workflow verified ({} admitted, {} selected, {} controls)",
                summary.admitted_actions,
                summary.frontier_selected,
                summary.negative_controls,
            );
        }
        Err(error) => {
            eprintln!("facade-kernel-operator-workflow: {error}");
            std::process::exit(1);
        }
    }
}
