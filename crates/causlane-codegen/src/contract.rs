//! Codegen contract surface (§7.10, §7.11).
//!
//! Names the Formal IR builder + per-target generators (§7.10) and the
//! stale-checker + coverage reporter (§7.11) as explicit trait contracts over
//! the crate's existing pure functions. Generators are pure (IR in, artifact
//! out, no filesystem writes — the CLI owns I/O); coverage is derived from
//! receipts and never upgrades `fail` to `pass`.
//!
//! Note on Alloy: the P/Kani/Verus generators consume the Formal IR and so
//! implement [`FormalGenerator`] directly. The Alloy lane
//! ([`crate::generate_alloy_facts_with_scenario`]) consumes the compiled bundle
//! plus scenario and builds the IR internally, so it does not fit the IR-only
//! `generate(&FormalIr)` shape and is intentionally not a `FormalGenerator`.

use causlane_contracts::CompiledDispatchBundle;

use crate::{
    build_formal_ir, build_report, generate_kani_harness, generate_lean4_proof, generate_p_monitor,
    generate_verus_proof, stale_check_with_expected, AlloyScenarioFacts, ArtifactFacts,
    CodegenError, CoverageMeta, FormalCoverageReport, FormalIr, FormalTarget, GeneratedArtifact,
    InvariantCoverage, NegativeControl,
};

/// §7.10 — build the Formal IR from a compiled bundle and optional scenario.
pub trait FormalIrBuilder {
    /// Build the Formal IR.
    ///
    /// # Errors
    /// Returns [`CodegenError`] if a predicate or scenario cannot be projected.
    fn build_ir(
        &self,
        bundle: &CompiledDispatchBundle,
        scenario: Option<&AlloyScenarioFacts>,
    ) -> Result<FormalIr, CodegenError>;
}

/// §7.10 — a pure per-target artifact generator. The generator never writes to
/// disk; it returns a [`GeneratedArtifact`] the CLI persists.
pub trait FormalGenerator {
    /// The formal target this generator emits.
    fn target(&self) -> FormalTarget;

    /// Generate the artifact for `ir`.
    ///
    /// # Errors
    /// Returns [`CodegenError`] if the artifact cannot be generated.
    fn generate(&self, ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError>;
}

/// §7.11 — stale-check a generated artifact against its source bundle, comparing
/// bundle/scenario/IR/artifact hashes recorded in the header and receipt.
pub trait StaleChecker {
    /// Validate that `generated_text` is fresh for `bundle` (and `receipt_json`
    /// / `expected_scenario_hash` when supplied).
    ///
    /// # Errors
    /// Returns [`CodegenError::Stale`] on any hash mismatch or
    /// [`CodegenError::Receipt`] on malformed receipt JSON.
    fn stale_check(
        &self,
        bundle: &CompiledDispatchBundle,
        generated_text: &str,
        receipt_json: Option<&str>,
        expected_scenario_hash: Option<&str>,
    ) -> Result<(), CodegenError>;
}

/// §7.11 — derive a coverage report from tool-run receipts. The report is
/// derived, not patched: it can never upgrade a `fail` lane to `pass`.
pub trait CoverageReporter {
    /// Build the coverage report from artifact facts and negative-control
    /// verdicts.
    fn build_coverage_report(
        &self,
        meta: &CoverageMeta,
        artifacts: &[ArtifactFacts],
        declared_invariants: &[InvariantCoverage],
        negative_controls: Vec<NegativeControl>,
    ) -> FormalCoverageReport;
}

/// The canonical codegen contract authority, delegating to the crate's pure
/// functions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CodegenContracts;

impl FormalIrBuilder for CodegenContracts {
    fn build_ir(
        &self,
        bundle: &CompiledDispatchBundle,
        scenario: Option<&AlloyScenarioFacts>,
    ) -> Result<FormalIr, CodegenError> {
        build_formal_ir(bundle, scenario)
    }
}

impl StaleChecker for CodegenContracts {
    fn stale_check(
        &self,
        bundle: &CompiledDispatchBundle,
        generated_text: &str,
        receipt_json: Option<&str>,
        expected_scenario_hash: Option<&str>,
    ) -> Result<(), CodegenError> {
        stale_check_with_expected(bundle, generated_text, receipt_json, expected_scenario_hash)
    }
}

impl CoverageReporter for CodegenContracts {
    fn build_coverage_report(
        &self,
        meta: &CoverageMeta,
        artifacts: &[ArtifactFacts],
        declared_invariants: &[InvariantCoverage],
        negative_controls: Vec<NegativeControl>,
    ) -> FormalCoverageReport {
        build_report(meta, artifacts, declared_invariants, negative_controls)
    }
}

/// §7.10 — the P protocol-monitor generator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PGenerator;

impl FormalGenerator for PGenerator {
    fn target(&self) -> FormalTarget {
        FormalTarget::P
    }

    fn generate(&self, ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
        generate_p_monitor(ir)
    }
}

/// §7.10 — the Kani bounded-harness generator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KaniGenerator;

impl FormalGenerator for KaniGenerator {
    fn target(&self) -> FormalTarget {
        FormalTarget::Kani
    }

    fn generate(&self, ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
        generate_kani_harness(ir)
    }
}

/// §7.10 — the Verus proof generator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VerusGenerator;

impl FormalGenerator for VerusGenerator {
    fn target(&self) -> FormalTarget {
        FormalTarget::Verus
    }

    fn generate(&self, ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
        generate_verus_proof(ir)
    }
}

/// §7.10 — the Lean4 theorem-application generator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lean4Generator;

impl FormalGenerator for Lean4Generator {
    fn target(&self) -> FormalTarget {
        FormalTarget::Lean4
    }

    fn generate(&self, ir: &FormalIr) -> Result<GeneratedArtifact, CodegenError> {
        generate_lean4_proof(ir)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CodegenContracts, CoverageReporter, FormalGenerator, FormalIrBuilder, KaniGenerator,
        Lean4Generator, PGenerator, StaleChecker, VerusGenerator,
    };
    use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};

    use crate::{
        default_invariant_matrix, generate_alloy_facts, CoverageMeta, FormalTarget, ReportStatus,
    };

    const REGISTRY: &str =
        include_str!("../fixtures/contracts/examples/release_promote.registry.yaml");

    /// Typed error union so `?` composes the crate error types in tests.
    #[derive(Debug)]
    #[allow(dead_code)]
    enum TestError {
        Contract(ContractError),
        Codegen(crate::CodegenError),
    }

    impl From<ContractError> for TestError {
        fn from(err: ContractError) -> Self {
            TestError::Contract(err)
        }
    }
    impl From<crate::CodegenError> for TestError {
        fn from(err: crate::CodegenError) -> Self {
            TestError::Codegen(err)
        }
    }

    type TestResult = Result<(), TestError>;

    fn demo_bundle() -> Result<CompiledDispatchBundle, ContractError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        CompiledDispatchBundle::compile(&manifest)
    }

    // §7.10: the IR builder + per-target generators round-trip; each generator
    // reports and emits its own target.
    #[test]
    fn ir_builder_and_generators_are_target_bound() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        for (generator_target, artifact) in [
            (PGenerator.target(), PGenerator.generate(&ir)?),
            (KaniGenerator.target(), KaniGenerator.generate(&ir)?),
            (VerusGenerator.target(), VerusGenerator.generate(&ir)?),
            (Lean4Generator.target(), Lean4Generator.generate(&ir)?),
        ] {
            assert_eq!(generator_target, artifact.target);
            assert!(artifact.artifact_hash.starts_with("sha256:"));
        }
        assert_eq!(PGenerator.target(), FormalTarget::P);
        Ok(())
    }

    // P0-006: every declared per-invariant check obligation for the IR-only
    // generators must literally appear in the freshly generated artifact, so the
    // coverage tables can never claim a lane the generator does not actually
    // emit. A renamed/removed check makes the present obligations diverge from
    // the candidate table and fails this test. (Alloy's payload-bound checks are
    // covered at the cli level where the real scenario projection is available.)
    #[test]
    fn obligations_match_generators() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        let cases = [
            (FormalTarget::P, PGenerator.generate(&ir)?),
            (FormalTarget::Kani, KaniGenerator.generate(&ir)?),
            (FormalTarget::Verus, VerusGenerator.generate(&ir)?),
            (FormalTarget::Lean4, Lean4Generator.generate(&ir)?),
        ];
        for (target, artifact) in cases {
            let mut present = artifact.obligations.clone();
            let mut expected = crate::obligations::all_obligations(target);
            let key = |o: &crate::ReceiptObligation| (o.invariant_id.clone(), o.check_id.clone());
            present.sort_by_key(key);
            expected.sort_by_key(key);
            assert_eq!(
                present, expected,
                "{target:?} obligation table drifted from the generator output"
            );
        }
        Ok(())
    }

    #[test]
    fn lean4_generator_emits_predicate_route_profile_bindings() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        let artifact = Lean4Generator.generate(&ir)?;
        assert!(artifact
            .text
            .contains("def generatedPredicateRoutes : List PredicateRoute"));
        assert!(artifact
            .text
            .contains("routeId := \"route.runtime_execution\""));
        assert!(artifact
            .text
            .contains("consequenceProfile := ConsequenceProfile.runtimeExecution"));
        assert!(artifact
            .text
            .contains("lifecycleClass := LifecycleClass.executionBearing"));
        assert!(artifact
            .text
            .contains("allRoutesConsistentWithProfiles generatedPredicateRoutes = true"));
        Ok(())
    }

    #[test]
    fn lean4_generator_emits_overlay_monotonicity_theorem() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        let artifact = Lean4Generator.generate(&ir)?;
        assert!(artifact.text.contains("theorem overlay_monotonicity"));
        assert!(artifact.text.contains("overlayMonotonicityHolds = true"));
        Ok(())
    }

    #[test]
    fn lean4_generator_emits_lease_conflict_theorems() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        let artifact = Lean4Generator.generate(&ir)?;
        assert!(artifact.text.contains("theorem lease_conflict_fail_closed"));
        assert!(artifact
            .text
            .contains("leaseConflictFailClosedHolds = true"));
        assert!(artifact.text.contains("theorem verified_merge_algebra"));
        assert!(artifact
            .text
            .contains("verifiedMergeClearsConflictHolds = true"));
        Ok(())
    }

    #[test]
    fn lean4_generator_emits_drain_theorem() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        let artifact = Lean4Generator.generate(&ir)?;
        assert!(artifact.text.contains("theorem drain_after_overlap_clear"));
        assert!(artifact.text.contains("drainAfterOverlapClearHolds = true"));
        Ok(())
    }

    #[test]
    fn lean4_generator_emits_constraint_update_theorem() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        let artifact = Lean4Generator.generate(&ir)?;
        assert!(artifact
            .text
            .contains("theorem constraint_update_future_only"));
        assert!(artifact
            .text
            .contains("constraintUpdateFutureOnlyHolds = true"));
        Ok(())
    }

    // §7.11 + H4: stale-check fails closed without a receipt (editable headers are
    // not sufficient proof of freshness); an artifact whose header declares a
    // different source bundle is also rejected as stale. The fresh+matching-receipt
    // happy path is covered by `tests::stale_check_accepts_a_matching_receipt`.
    #[test]
    fn stale_checker_detects_bundle_drift() -> TestResult {
        let bundle = demo_bundle()?;
        let artifact = generate_alloy_facts(&bundle, None)?;
        let checker = CodegenContracts;
        // No receipt -> fail closed (H4).
        assert!(checker
            .stale_check(&bundle, &artifact.text, None, None)
            .is_err());
        // A header declaring a different source bundle is stale (caught by the
        // source_bundle_hash header check, before the receipt requirement).
        let drifted = artifact.text.replace(
            &bundle.bundle_hash.0,
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert!(checker.stale_check(&bundle, &drifted, None, None).is_err());
        Ok(())
    }

    // §7.11: coverage is derived from the receipts/invariant matrix and carries
    // the source bundle hash; an empty-artifact report is a typed report value.
    #[test]
    fn coverage_reporter_derives_report() -> TestResult {
        let bundle = demo_bundle()?;
        let ir = CodegenContracts.build_ir(&bundle, None)?;
        let meta = CoverageMeta {
            checked_at: "1970-01-01T00:00:00Z".to_owned(),
            source_bundle_hash: ir.source_bundle_hash.clone(),
            formal_ir_hash: ir.formal_ir_hash.clone(),
            scenario_hash: None,
            formal_ir_path: "formal/ir.json".to_owned(),
        };
        let report = CodegenContracts.build_coverage_report(
            &meta,
            &[],
            &default_invariant_matrix(),
            Vec::new(),
        );
        assert_eq!(report.source_bundle_hash, ir.source_bundle_hash);
        assert!(!report.invariant_coverage.is_empty());
        assert!(matches!(
            report.status,
            ReportStatus::Pass | ReportStatus::Fail
        ));
        Ok(())
    }
}
