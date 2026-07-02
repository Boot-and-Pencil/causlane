//! Formal verification orchestration helper.

#![forbid(unsafe_code)]
#![deny(warnings)]

use std::process::ExitCode;

#[path = "formal_runtime/io.rs"]
mod formal_runtime_io;

use causlane_cli::app::formal::{
    CoverageRequest, FormalOrchestrator, FormalServiceError, GenerateAllRequest,
    StaleCheckAllRequest, VerifyAllRequest,
};
use causlane_cli::cli_shared::{
    flag_value, DEFAULT_FORMAL_ARTIFACT_DIR, DEFAULT_FORMAL_COVERAGE_REPORT,
    DEFAULT_FORMAL_RECEIPT_DIR,
};

use formal_runtime_io::StdFormalIo;

/// Shared flags for both sub-commands.
struct Args {
    bundle: String,
    scenario: String,
    artifact_dir: String,
    receipt_dir: String,
    coverage: String,
}

enum Command {
    /// Generate artifacts + receipts, then write a derived preliminary report.
    VerifyAll(Args),
    /// Re-derive the coverage report from on-disk tool-run receipts.
    Coverage(Args),
    /// Generate all formal targets and codegen receipts.
    GenerateAll(Args),
    /// Check all generated targets and codegen receipts for freshness.
    StaleCheckAll(Args),
}

fn parse(args: &[String]) -> Option<Command> {
    let sub = args.get(1)?;
    let parsed = Args {
        bundle: flag_value(args, "--bundle")?,
        scenario: flag_value(args, "--scenario")?,
        artifact_dir: flag_value(args, "--artifact-dir")
            .unwrap_or_else(|| DEFAULT_FORMAL_ARTIFACT_DIR.to_owned()),
        receipt_dir: flag_value(args, "--receipt-dir")
            .unwrap_or_else(|| DEFAULT_FORMAL_RECEIPT_DIR.to_owned()),
        coverage: flag_value(args, "--coverage")
            .unwrap_or_else(|| DEFAULT_FORMAL_COVERAGE_REPORT.to_owned()),
    };
    match sub.as_str() {
        "verify-all" => Some(Command::VerifyAll(parsed)),
        "coverage" => Some(Command::Coverage(parsed)),
        "generate-all" => Some(Command::GenerateAll(parsed)),
        "stale-check-all" => Some(Command::StaleCheckAll(parsed)),
        _ => None,
    }
}

fn usage() -> String {
    [
        "causlane-formal helper",
        "usage:",
        "  causlane-formal verify-all --bundle <bundle.json> --scenario <scenario.yaml> [--artifact-dir verification/formal-full] [--receipt-dir verification/formal-full/receipts] [--coverage target/causlane/formal-coverage-report.json]",
        "  causlane-formal coverage   --bundle <bundle.json> --scenario <scenario.yaml> [--artifact-dir verification/formal-full] [--receipt-dir verification/formal-full/receipts] [--coverage target/causlane/formal-coverage-report.json]",
        "  causlane-formal generate-all --bundle <bundle.json> --scenario <scenario.yaml> [--artifact-dir verification/formal-full] [--receipt-dir verification/formal-full/receipts]",
        "  causlane-formal stale-check-all --bundle <bundle.json> --scenario <scenario.yaml> [--artifact-dir verification/formal-full] [--receipt-dir verification/formal-full/receipts]",
        "",
        "verify-all generates artifacts + receipts and writes a derived (preliminary) coverage report.",
        "coverage re-derives the report from the tool-run receipts on disk after the tools have run.",
        "generate-all emits Formal IR, all generated targets and codegen receipts.",
        "stale-check-all verifies all generated targets against bundle, scenario and receipts.",
    ]
    .join("\n")
}

fn run(args: &[String]) -> Result<String, FormalServiceError> {
    let mut io = StdFormalIo;
    match parse(args) {
        Some(Command::VerifyAll(parsed)) => FormalOrchestrator.verify_all(
            &mut io,
            VerifyAllRequest {
                bundle_path: &parsed.bundle,
                scenario_path: &parsed.scenario,
                artifact_dir: &parsed.artifact_dir,
                receipt_dir: &parsed.receipt_dir,
                coverage_path: &parsed.coverage,
            },
        ),
        Some(Command::Coverage(parsed)) => FormalOrchestrator.coverage(
            &mut io,
            CoverageRequest {
                bundle_path: &parsed.bundle,
                scenario_path: &parsed.scenario,
                artifact_dir: &parsed.artifact_dir,
                receipt_dir: &parsed.receipt_dir,
                coverage_path: &parsed.coverage,
            },
        ),
        Some(Command::GenerateAll(parsed)) => FormalOrchestrator.generate_all(
            &mut io,
            GenerateAllRequest {
                bundle_path: &parsed.bundle,
                scenario_path: Some(&parsed.scenario),
                artifact_dir: &parsed.artifact_dir,
                receipt_dir: &parsed.receipt_dir,
            },
        ),
        Some(Command::StaleCheckAll(parsed)) => FormalOrchestrator.stale_check_all(
            &mut io,
            StaleCheckAllRequest {
                bundle_path: &parsed.bundle,
                scenario_path: Some(&parsed.scenario),
                artifact_dir: &parsed.artifact_dir,
                receipt_dir: &parsed.receipt_dir,
            },
        ),
        None => Err(FormalServiceError::Usage(usage())),
    }
}

fn main() -> ExitCode {
    let args = std::env::args().collect::<Vec<_>>();
    match run(&args) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use causlane_contracts::{CompiledDispatchBundle, ContractError, RegistryManifest};
    use std::path::{Path, PathBuf};

    const REGISTRY: &str =
        include_str!("../../fixtures/contracts/examples/release_promote.registry.yaml");

    #[derive(Debug)]
    enum TestError {
        Contract,
        Formal,
        Io,
        Missing,
    }

    impl From<ContractError> for TestError {
        fn from(_err: ContractError) -> Self {
            Self::Contract
        }
    }

    impl From<FormalServiceError> for TestError {
        fn from(_err: FormalServiceError) -> Self {
            Self::Formal
        }
    }

    impl From<std::io::Error> for TestError {
        fn from(_err: std::io::Error) -> Self {
            Self::Io
        }
    }

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    fn write_demo_bundle(path: &Path) -> Result<(), TestError> {
        let manifest = RegistryManifest::from_yaml_str(REGISTRY)?;
        let bundle = CompiledDispatchBundle::compile(&manifest)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bundle.to_json_pretty()?)?;
        Ok(())
    }

    #[test]
    fn verify_all_then_coverage_rederive_writes_report() -> Result<(), TestError> {
        let root = PathBuf::from("../../target/causlane-cli-tests/causlane-formal");
        let bundle = root.join("release_promote.bundle.json");
        let artifacts = root.join("verification/formal-full").display().to_string();
        let receipts = root.join("receipts").display().to_string();
        let coverage = root.join("coverage.json").display().to_string();
        let scenario = "../../contracts/scenarios/release_promote_success.scenario.yaml";
        write_demo_bundle(&bundle)?;
        let bundle_path = bundle.display().to_string();
        let common = [
            "--bundle",
            &bundle_path,
            "--scenario",
            scenario,
            "--artifact-dir",
            &artifacts,
            "--receipt-dir",
            &receipts,
            "--coverage",
            &coverage,
        ];

        let mut verify = vec!["causlane-formal", "verify-all"];
        verify.extend(common);
        if !run(&args(&verify))?.contains("formal artifacts generated") {
            return Err(TestError::Missing);
        }

        let mut rederive = vec!["causlane-formal", "coverage"];
        rederive.extend(common);
        if !run(&args(&rederive))?.contains("coverage derived from receipts") {
            return Err(TestError::Missing);
        }
        if !std::fs::read_to_string(&coverage)?.contains("\"invariant_coverage\"") {
            return Err(TestError::Missing);
        }
        Ok(())
    }

    #[test]
    fn generate_all_then_stale_check_all_uses_shared_service() -> Result<(), TestError> {
        let root = PathBuf::from("../../target/causlane-cli-tests/causlane-formal-generate-all");
        let bundle = root.join("release_promote.bundle.json");
        let artifacts = root.join("verification/formal-full").display().to_string();
        let receipts = root.join("receipts").display().to_string();
        let scenario = "../../contracts/scenarios/release_promote_success.scenario.yaml";
        write_demo_bundle(&bundle)?;
        let bundle_path = bundle.display().to_string();
        let common = [
            "--bundle",
            &bundle_path,
            "--scenario",
            scenario,
            "--artifact-dir",
            &artifacts,
            "--receipt-dir",
            &receipts,
        ];

        let mut generate = vec!["causlane-formal", "generate-all"];
        generate.extend(common);
        if !run(&args(&generate))?.contains("generated all targets") {
            return Err(TestError::Missing);
        }

        let mut stale = vec!["causlane-formal", "stale-check-all"];
        stale.extend(common);
        if !run(&args(&stale))?.contains("all generated targets fresh") {
            return Err(TestError::Missing);
        }
        Ok(())
    }
}
