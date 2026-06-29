#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_contracts_boundary_ergonomics_example::run_contracts_boundary() {
        Ok(summary) => {
            println!(
                "contracts-boundary-ergonomics: registry bundle and plan hashes verified ({} bundle, {} hashes, {} controls)",
                summary.compiled_bundles, summary.canonical_hashes, summary.negative_controls,
            );
        }
        Err(error) => {
            eprintln!("contracts-boundary-ergonomics: {error}");
            std::process::exit(1);
        }
    }
}
