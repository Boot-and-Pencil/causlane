#![forbid(unsafe_code)]
#![deny(warnings)]

use causlane_contracts::{BoundaryContracts, BundleCompiler, RegistryManifest};
use causlane_contracts_boundary_ergonomics_example::{
    run_contracts_boundary, verify_missing_template_fails,
    verify_plan_material_mutation_changes_hash, verify_tampered_bundle_is_rejected,
};

const REGISTRY_YAML: &str =
    include_str!("../../../contracts/examples/release_promote.registry.yaml");

#[test]
fn contracts_boundary_summary_counts_positive_and_negative_checks(
) -> Result<(), Box<dyn std::error::Error>> {
    let summary = run_contracts_boundary()?;
    assert_eq!(summary.parsed_manifests, 1);
    assert_eq!(summary.compiled_bundles, 1);
    assert_eq!(summary.verified_artifacts, 1);
    assert_eq!(summary.canonical_hashes, 2);
    assert_eq!(summary.resolved_templates, 2);
    assert_eq!(summary.negative_controls, 3);
    Ok(())
}

#[test]
fn contracts_boundary_negative_controls_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let boundary = BoundaryContracts;
    let manifest = RegistryManifest::from_yaml_str(REGISTRY_YAML)?;
    let bundle = boundary.compile_registry(&manifest)?;

    assert_eq!(verify_tampered_bundle_is_rejected(&bundle)?, 1);
    assert_eq!(verify_missing_template_fails(&boundary)?, 1);
    assert_eq!(verify_plan_material_mutation_changes_hash(&bundle)?, 1);
    Ok(())
}
