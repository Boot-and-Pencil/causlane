//! causlane developer/operator CLI.
//!
//! See `causlane --help` / [`cli_parse::usage`] for command forms.
//!
//! This binary is the platform boundary: environment/file I/O (PATH scanning,
//! reading files) lives here, while the pure logic lives in the libraries.

#![forbid(unsafe_code)]
#![deny(warnings)]

use core::fmt;
use std::path::Path;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

mod cli_contract;
mod cli_explain;
mod cli_graph;
mod cli_graph_export;
mod cli_parse;
mod cli_replay;
mod cli_scenario;
mod cli_support_bundle;
mod formal_generate;
#[path = "bin/formal_runtime/io.rs"]
mod formal_runtime_io;

use cli_parse::{parse, usage};

use causlane_cli::app::formal::FormalServiceError;
use causlane_cli::cli_shared::{
    checked_at_token_from_unix_secs, fallback_checked_at_token, read_file_with, write_file_with,
    FileError,
};
use causlane_codegen::{CodegenError, FormalTarget};
use causlane_contracts::{
    BoundaryContracts, BundleCompiler, CompiledDispatchBundle, ContractError, RegistryManifest,
};
use causlane_replay::{ReplayError, ReplayScenario};
pub(crate) use formal_runtime_io::StdFormalIo;

/// A parsed CLI command (boundary strings resolved before branching).
enum Command {
    BundleValidate(String),
    BundleCompile {
        registry: String,
        out: String,
    },
    ReplayVerifyStructural(String),
    ReplayVerifyWithBundle {
        bundle: String,
        trace: String,
        require_bundle_hash: bool,
        kernel_secret: Option<String>,
        explain: bool,
        json: bool,
    },
    ExplainReplay {
        bundle: String,
        trace: String,
        json: bool,
    },
    WhyBlocked {
        graph: String,
        op: String,
        json: bool,
    },
    WhyNotParallel {
        graph: String,
        op: String,
        with: String,
        json: bool,
    },
    GraphExport {
        graph: String,
        format: String,
        op: Option<String>,
        out: Option<String>,
    },
    SupportBundleBuild {
        bundle: String,
        trace: String,
        graph: String,
        out: String,
        op: Option<String>,
    },
    ScenarioEmitTrace {
        scenario: String,
        out: String,
        bundle: Option<String>,
        kernel_secret: Option<String>,
    },
    ScenarioCompile {
        scenario: String,
        bundle: String,
        out_dir: String,
        kernel_secret: Option<String>,
    },
    ScenarioValidate(String),
    ContractTest {
        manifest: String,
        json: bool,
    },
    FormalGenerateAlloy {
        bundle: String,
        out: String,
        scenario: Option<String>,
        receipt: Option<String>,
    },
    FormalStaleCheck {
        bundle: String,
        generated: String,
        scenario: Option<String>,
        receipt: Option<String>,
    },
    FormalGenerate {
        target: FormalTarget,
        bundle: String,
        scenario: Option<String>,
        out: String,
        receipt: Option<String>,
    },
    FormalGenerateAll {
        bundle: String,
        scenario: Option<String>,
        artifact_dir: String,
        receipt_dir: String,
    },
    FormalStaleCheckAll {
        bundle: String,
        scenario: Option<String>,
        artifact_dir: String,
        receipt_dir: String,
    },
    FormalIrEmit {
        bundle: String,
        scenario: Option<String>,
        out: String,
    },
}

/// Typed CLI error.
#[derive(Debug)]
enum CliError {
    File(FileError),
    Contract(ContractError),
    Codegen(CodegenError),
    Replay(ReplayError),
    Formal(FormalServiceError),
    Usage(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::File(err) => write!(f, "{err}"),
            CliError::Contract(err) => write!(f, "{err}"),
            CliError::Codegen(err) => write!(f, "{err}"),
            CliError::Replay(err) => write!(f, "{err}"),
            CliError::Formal(err) => write!(f, "{err}"),
            CliError::Usage(text) => write!(f, "{text}"),
        }
    }
}

impl From<FileError> for CliError {
    fn from(err: FileError) -> Self {
        CliError::File(err)
    }
}

impl From<ContractError> for CliError {
    fn from(err: ContractError) -> Self {
        CliError::Contract(err)
    }
}

impl From<ReplayError> for CliError {
    fn from(err: ReplayError) -> Self {
        CliError::Replay(err)
    }
}

impl From<CodegenError> for CliError {
    fn from(err: CodegenError) -> Self {
        CliError::Codegen(err)
    }
}

impl From<FormalServiceError> for CliError {
    fn from(err: FormalServiceError) -> Self {
        CliError::Formal(err)
    }
}

/// Output of a command: text plus whether it counts as success (exit 0).
struct RunOutput {
    text: String,
    success: bool,
}

fn read_file(path: &str) -> Result<String, CliError> {
    Ok(read_file_with(path, |path: &Path| {
        std::fs::read_to_string(path)
    })?)
}

fn write_file(path: impl AsRef<Path>, content: &str) -> Result<(), CliError> {
    Ok(write_file_with(
        path,
        content,
        |path: &Path| std::fs::create_dir_all(path),
        |path: &Path, content| std::fs::write(path, content),
    )?)
}

fn compile_bundle_from_registry(path: &str) -> Result<CompiledDispatchBundle, CliError> {
    let yaml = read_file(path)?;
    let manifest = RegistryManifest::from_yaml_str(&yaml)?;
    Ok(BoundaryContracts.compile_registry(&manifest)?)
}

fn read_bundle(path: &str) -> Result<CompiledDispatchBundle, CliError> {
    let json = read_file(path)?;
    Ok(CompiledDispatchBundle::from_json_str(&json)?)
}

fn checked_at_token() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => checked_at_token_from_unix_secs(duration.as_secs()),
        Err(_err) => fallback_checked_at_token(),
    }
}

fn validate_bundle(path: &str) -> Result<String, CliError> {
    let bundle = compile_bundle_from_registry(path)?;
    Ok(format!(
        "ok: compiled {} predicate(s); bundle_hash {}",
        bundle.body.predicates.len(),
        bundle.bundle_hash.0
    ))
}

fn compile_bundle(path: &str, out: &str) -> Result<String, CliError> {
    let bundle = compile_bundle_from_registry(path)?;
    let json = bundle.to_json_pretty()?;
    write_file(out, &json)?;
    Ok(format!(
        "ok: wrote {out}; bundle_hash {}",
        bundle.bundle_hash.0
    ))
}

fn validate_scenario_file(path: &str) -> Result<String, CliError> {
    let yaml = read_file(path)?;
    // Parse through the typed scenario DTO (the source of truth): a malformed
    // event kind, claim mode, missing required field or bad type is rejected here.
    let scenario = ReplayScenario::from_yaml_str(&yaml)?;
    Ok(format!(
        "ok: {path} is a valid scenario ({} events)",
        scenario.events.len()
    ))
}

#[allow(clippy::too_many_lines)]
fn run(args: &[String]) -> Result<RunOutput, CliError> {
    match parse(args) {
        Some(Command::BundleValidate(path)) => Ok(RunOutput {
            text: validate_bundle(&path)?,
            success: true,
        }),
        Some(Command::BundleCompile { registry, out }) => Ok(RunOutput {
            text: compile_bundle(&registry, &out)?,
            success: true,
        }),
        Some(Command::ReplayVerifyStructural(path)) => Ok(RunOutput {
            text: cli_replay::verify_trace(&path)?,
            success: true,
        }),
        Some(Command::ReplayVerifyWithBundle {
            bundle,
            trace,
            require_bundle_hash,
            kernel_secret,
            explain,
            json,
        }) => {
            if explain {
                cli_replay::explain_trace_with_bundle(&bundle, &trace, json)
            } else {
                Ok(RunOutput {
                    text: cli_replay::verify_trace_with_bundle(
                        &bundle,
                        &trace,
                        require_bundle_hash,
                        kernel_secret.as_deref(),
                    )?,
                    success: true,
                })
            }
        }
        Some(Command::ExplainReplay {
            bundle,
            trace,
            json,
        }) => cli_replay::explain_trace_with_bundle(&bundle, &trace, json),
        Some(Command::WhyBlocked { graph, op, json }) => {
            cli_explain::why_blocked(&graph, &op, json)
        }
        Some(Command::WhyNotParallel {
            graph,
            op,
            with,
            json,
        }) => cli_explain::why_not_parallel(&graph, &op, &with, json),
        Some(Command::GraphExport {
            graph,
            format,
            op,
            out,
        }) => cli_graph_export::export_graph(&graph, &format, op.as_deref(), out.as_deref()),
        Some(Command::SupportBundleBuild {
            bundle,
            trace,
            graph,
            out,
            op,
        }) => {
            cli_support_bundle::build_support_bundle(&bundle, &trace, &graph, &out, op.as_deref())
        }
        Some(Command::ScenarioEmitTrace {
            scenario,
            out,
            bundle,
            kernel_secret,
        }) => Ok(RunOutput {
            text: cli_scenario::emit_trace_from_scenario(
                &scenario,
                &out,
                bundle.as_deref(),
                kernel_secret.as_deref(),
            )?,
            success: true,
        }),
        Some(Command::ScenarioCompile {
            scenario,
            bundle,
            out_dir,
            kernel_secret,
        }) => Ok(RunOutput {
            text: cli_scenario::compile_scenario_to_dir(
                &scenario,
                &bundle,
                &out_dir,
                kernel_secret.as_deref(),
            )?,
            success: true,
        }),
        Some(Command::ScenarioValidate(path)) => Ok(RunOutput {
            text: validate_scenario_file(&path)?,
            success: true,
        }),
        Some(Command::ContractTest { manifest, json }) => {
            cli_contract::run_contract_tests(&manifest, json)
        }
        Some(Command::FormalGenerateAlloy {
            bundle,
            out,
            scenario,
            receipt,
        }) => Ok(RunOutput {
            text: formal_generate::generate_alloy(
                &bundle,
                &out,
                scenario.as_deref(),
                receipt.as_deref(),
            )?,
            success: true,
        }),
        Some(Command::FormalGenerate {
            target,
            bundle,
            scenario,
            out,
            receipt,
        }) => Ok(RunOutput {
            text: formal_generate::generate_single(
                target,
                &bundle,
                scenario.as_deref(),
                &out,
                receipt.as_deref(),
            )?,
            success: true,
        }),
        Some(Command::FormalGenerateAll {
            bundle,
            scenario,
            artifact_dir,
            receipt_dir,
        }) => Ok(RunOutput {
            text: formal_generate::generate_all(
                &bundle,
                scenario.as_deref(),
                &artifact_dir,
                &receipt_dir,
            )?,
            success: true,
        }),
        Some(Command::FormalStaleCheck {
            bundle,
            generated,
            scenario,
            receipt,
        }) => Ok(RunOutput {
            text: formal_generate::check_generated(
                &bundle,
                &generated,
                scenario.as_deref(),
                receipt.as_deref(),
            )?,
            success: true,
        }),
        Some(Command::FormalStaleCheckAll {
            bundle,
            scenario,
            artifact_dir,
            receipt_dir,
        }) => Ok(RunOutput {
            text: formal_generate::stale_check_all(
                &bundle,
                scenario.as_deref(),
                &artifact_dir,
                &receipt_dir,
            )?,
            success: true,
        }),
        Some(Command::FormalIrEmit {
            bundle,
            scenario,
            out,
        }) => Ok(RunOutput {
            text: formal_generate::emit_ir(&bundle, scenario.as_deref(), &out)?,
            success: true,
        }),
        None => Err(CliError::Usage(usage())),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(output) => {
            println!("{}", output.text);
            if output.success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod cli_smoke_tests;
