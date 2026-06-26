//! Shared formal artifact planning for `causlane` and `causlane-formal`.

use std::path::PathBuf;

use causlane_codegen::{
    generate_alloy_facts, generate_alloy_facts_with_scenario, AlloyScenarioFacts, CodegenError,
    FormalGenerator, FormalIr, FormalTarget, GeneratedArtifact, KaniGenerator, Lean4Generator,
    PGenerator, VerusGenerator,
};
use causlane_contracts::CompiledDispatchBundle;

/// All formal targets in the stable generation/check order.
pub const FORMAL_TARGETS: [FormalTarget; 5] = [
    FormalTarget::Alloy,
    FormalTarget::P,
    FormalTarget::Kani,
    FormalTarget::Verus,
    FormalTarget::Lean4,
];

/// Generated artifact plus the paths where its outputs and receipts live.
pub struct FormalArtifactPlan {
    /// In-memory generated artifact.
    pub artifact: GeneratedArtifact,
    /// Output path for the generated artifact.
    pub artifact_path: PathBuf,
    /// Output path for the codegen receipt.
    pub codegen_receipt_path: PathBuf,
    /// Output path for the tool-run receipt.
    pub tool_run_receipt_path: PathBuf,
}

/// Map a single-target CLI token to a formal target.
#[must_use]
pub fn single_target_from_kind(kind: &str) -> Option<FormalTarget> {
    if kind == "p" {
        Some(FormalTarget::P)
    } else if kind == "kani" {
        Some(FormalTarget::Kani)
    } else if kind == "verus" {
        Some(FormalTarget::Verus)
    } else if kind == "lean4" {
        Some(FormalTarget::Lean4)
    } else {
        None
    }
}

/// Return the generated artifact file extension for a target.
#[must_use]
pub fn target_extension(target: FormalTarget) -> &'static str {
    match target {
        FormalTarget::Alloy => "als",
        FormalTarget::P => "p",
        FormalTarget::Kani | FormalTarget::Verus => "rs",
        FormalTarget::Lean4 => "lean",
    }
}

/// Return the generated artifact path for a target and scenario stem.
#[must_use]
pub fn artifact_path_for(artifact_dir: &str, target: FormalTarget, stem: &str) -> PathBuf {
    PathBuf::from(artifact_dir)
        .join(target.as_str())
        .join("generated")
        .join(format!("{stem}.{}", target_extension(target)))
}

/// Return the generated Formal IR path for a scenario stem.
#[must_use]
pub fn ir_path_for(artifact_dir: &str, stem: &str) -> PathBuf {
    PathBuf::from(artifact_dir)
        .join("ir")
        .join("generated")
        .join(format!("{stem}.formal_ir.json"))
}

/// Return the codegen receipt path for a target and scenario stem.
#[must_use]
pub fn codegen_receipt_path_for(receipt_dir: &str, target: FormalTarget, stem: &str) -> PathBuf {
    PathBuf::from(receipt_dir).join(format!("{stem}.{}.codegen.json", target.as_str()))
}

/// Return the tool-run receipt path for a target and scenario stem.
#[must_use]
pub fn tool_run_receipt_path_for(receipt_dir: &str, target: FormalTarget, stem: &str) -> PathBuf {
    PathBuf::from(receipt_dir).join(format!("{stem}.{}.tool-run.json", target.as_str()))
}

/// Generate one formal artifact for the selected target.
#[must_use = "generated artifacts must be written or checked by the caller"]
pub fn artifact_for(
    target: FormalTarget,
    bundle: &CompiledDispatchBundle,
    ir: &FormalIr,
    facts: Option<&AlloyScenarioFacts>,
) -> Result<GeneratedArtifact, CodegenError> {
    let artifact = match target {
        FormalTarget::Alloy => match facts {
            Some(facts) => generate_alloy_facts_with_scenario(bundle, facts)?,
            None => generate_alloy_facts(bundle, None)?,
        },
        FormalTarget::P => PGenerator.generate(ir)?,
        FormalTarget::Kani => KaniGenerator.generate(ir)?,
        FormalTarget::Verus => VerusGenerator.generate(ir)?,
        FormalTarget::Lean4 => Lean4Generator.generate(ir)?,
    };
    Ok(artifact)
}

/// Generate every formal artifact and compute their output paths.
#[must_use = "formal artifact plans must be written or checked by the caller"]
pub fn plan_artifacts(
    bundle: &CompiledDispatchBundle,
    facts: &AlloyScenarioFacts,
    ir: &FormalIr,
    artifact_dir: &str,
    receipt_dir: &str,
    stem: &str,
) -> Result<Vec<FormalArtifactPlan>, CodegenError> {
    FORMAL_TARGETS
        .into_iter()
        .map(|target| {
            let artifact = artifact_for(target, bundle, ir, Some(facts))?;
            Ok(FormalArtifactPlan {
                artifact,
                artifact_path: artifact_path_for(artifact_dir, target, stem),
                codegen_receipt_path: codegen_receipt_path_for(receipt_dir, target, stem),
                tool_run_receipt_path: tool_run_receipt_path_for(receipt_dir, target, stem),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        artifact_path_for, codegen_receipt_path_for, ir_path_for, single_target_from_kind,
        target_extension, tool_run_receipt_path_for,
    };
    use causlane_codegen::FormalTarget;

    #[test]
    fn target_kind_and_extensions_are_stable() {
        assert_eq!(single_target_from_kind("p"), Some(FormalTarget::P));
        assert_eq!(single_target_from_kind("alloy"), None);
        assert_eq!(target_extension(FormalTarget::Alloy), "als");
        assert_eq!(target_extension(FormalTarget::Verus), "rs");
    }

    #[test]
    fn formal_paths_share_one_layout() {
        let stem = "release_promote_success";
        assert_eq!(
            ir_path_for("formal", stem).display().to_string(),
            "formal/ir/generated/release_promote_success.formal_ir.json"
        );
        assert_eq!(
            artifact_path_for("formal", FormalTarget::Lean4, stem)
                .display()
                .to_string(),
            "formal/lean4/generated/release_promote_success.lean"
        );
        assert_eq!(
            codegen_receipt_path_for("formal/receipts", FormalTarget::P, stem)
                .display()
                .to_string(),
            "formal/receipts/release_promote_success.p.codegen.json"
        );
        assert_eq!(
            tool_run_receipt_path_for("formal/receipts", FormalTarget::Kani, stem)
                .display()
                .to_string(),
            "formal/receipts/release_promote_success.kani.tool-run.json"
        );
    }
}
