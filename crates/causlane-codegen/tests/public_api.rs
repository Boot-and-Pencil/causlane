//! Public API regression tests for crate-root re-exports.

use causlane_codegen::{
    generate_alloy_facts, AlloyScenarioFacts, CodegenError, FormalTarget, GeneratedArtifact,
    ReceiptScope, ToolRunResult,
};
use causlane_contracts::CompiledDispatchBundle;

fn assert_generator_type(
    _: fn(&CompiledDispatchBundle, Option<&str>) -> Result<GeneratedArtifact, CodegenError>,
) {
}

#[test]
fn crate_root_reexports_formal_api_types() {
    assert_eq!(FormalTarget::P.as_str(), "p");
    assert_eq!(ToolRunResult::Generated, ToolRunResult::Generated);
    assert_eq!(
        ReceiptScope {
            predicates: 1,
            scenarios: 0,
        },
        ReceiptScope {
            predicates: 1,
            scenarios: 0,
        }
    );
    assert!(CodegenError::Header("missing".to_owned())
        .to_string()
        .contains("generated header error"));

    assert_generator_type(generate_alloy_facts);
    let _ = std::any::type_name::<AlloyScenarioFacts>();
}
