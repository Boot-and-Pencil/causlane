#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_contracts_registry_bundle_workflow_example::run_contracts_registry_bundle_workflow(
    ) {
        Ok(summary) => {
            println!(
                "contracts-registry-bundle-workflow: registry bundle workflow verified ({} predicates, {} plan cache lookups, {} controls)",
                summary.compiled_predicates,
                summary.plan_cache_lookups,
                summary.negative_controls,
            );
        }
        Err(error) => {
            eprintln!("contracts-registry-bundle-workflow: {error}");
            std::process::exit(1);
        }
    }
}
