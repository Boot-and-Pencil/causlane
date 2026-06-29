#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_runtime_guarded_audit_projection_example::run_runtime_guarded_audit_projection()
    {
        Ok(summary) => {
            println!(
                "runtime-guarded-audit-projection: authz execution audit and projection verified ({} executed, {} audit events, {} controls)",
                summary.executed_ops, summary.audit_events, summary.negative_controls,
            );
        }
        Err(error) => {
            eprintln!("runtime-guarded-audit-projection: {error}");
            std::process::exit(1);
        }
    }
}
