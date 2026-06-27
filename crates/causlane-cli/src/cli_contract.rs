//! `contract test` — a data-driven YAML contract-testing harness (M04.5).
//!
//! A manifest lists suites of `{registry, scenarios}`. For each scenario the
//! harness compiles it against its suite's bundle (reusing `compile_scenario`,
//! M04.3) and replays it (`verify_explain`, M04.4), asserting the scenario's OWN
//! declared contract — `expected_replay_result` plus `expected_error_code` for
//! negatives. Fails closed: any mismatch (or a manifest/IO error) is a failure.
//! A new scenario gets contract coverage by adding it to the manifest — no Rust.

use serde::{Deserialize, Serialize};

use causlane_replay::{ExpectedReplayResult, ReplayExplain};
use noyalib::compat::serde_yaml;

use crate::cli_scenario::{compile_scenario, ScenarioExpectation};
use crate::{compile_bundle_from_registry, read_file, CliError, RunOutput};

#[derive(Debug, Deserialize)]
struct ContractManifest {
    suites: Vec<ContractSuite>,
}

#[derive(Debug, Deserialize)]
struct ContractSuite {
    name: String,
    registry: String,
    scenarios: Vec<String>,
}

/// One evaluated contract case.
#[derive(Debug, Serialize)]
struct CaseResult {
    suite: String,
    scenario: String,
    expected: String,
    actual: String,
    ok: bool,
}

/// The full machine-readable contract-test report.
#[derive(Debug, Serialize)]
struct ContractReport {
    total: usize,
    passed: usize,
    failed: usize,
    cases: Vec<CaseResult>,
}

/// `pass` for an accepted trace, `fail:CODE` for a rejected one.
fn observed_token(explain: &ReplayExplain) -> String {
    if explain.accepted {
        "pass".to_owned()
    } else {
        format!("fail:{}", explain.error_code.as_deref().unwrap_or("?"))
    }
}

/// The scenario's declared expectation as a token (`pass`, `fail`, `fail:CODE`).
fn expected_token(expectation: &ScenarioExpectation) -> String {
    match expectation.expected_result {
        ExpectedReplayResult::Pass => "pass".to_owned(),
        ExpectedReplayResult::Fail => match &expectation.expected_error_code {
            Some(code) => format!("fail:{code}"),
            None => "fail".to_owned(),
        },
    }
}

/// Whether the observed replay outcome satisfies the scenario's declared
/// contract. A negative scenario with a declared code must reject with exactly
/// that code; one without a declared code must merely reject.
fn contract_satisfied(expectation: &ScenarioExpectation, explain: &ReplayExplain) -> bool {
    match expectation.expected_result {
        ExpectedReplayResult::Pass => explain.accepted,
        ExpectedReplayResult::Fail => {
            !explain.accepted
                && expectation
                    .expected_error_code
                    .as_deref()
                    .is_none_or(|code| explain.error_code.as_deref() == Some(code))
        }
    }
}

/// Run every case in the manifest and return a report; exit success iff all pass.
pub(crate) fn run_contract_tests(manifest_path: &str, json: bool) -> Result<RunOutput, CliError> {
    let manifest_yaml = read_file(manifest_path)?;
    let manifest: ContractManifest = serde_yaml::from_str(&manifest_yaml)
        .map_err(|err| CliError::Usage(format!("invalid contract manifest: {err}")))?;

    let mut cases = Vec::new();
    for suite in &manifest.suites {
        let bundle = compile_bundle_from_registry(&suite.registry)?;
        for scenario_path in &suite.scenarios {
            let scenario_yaml = read_file(scenario_path)?;
            let compiled = compile_scenario(&scenario_yaml, &bundle, None)?;
            let explain = compiled.trace.verify_explain(&bundle);
            cases.push(CaseResult {
                suite: suite.name.clone(),
                scenario: scenario_path.clone(),
                expected: expected_token(&compiled.expectation),
                actual: observed_token(&explain),
                ok: contract_satisfied(&compiled.expectation, &explain),
            });
        }
    }

    let passed = cases.iter().filter(|case| case.ok).count();
    let report = ContractReport {
        total: cases.len(),
        passed,
        failed: cases.len() - passed,
        cases,
    };

    let text = if json {
        serde_json::to_string_pretty(&report)
            .map_err(|err| CliError::Usage(format!("cannot serialize report: {err}")))?
    } else {
        let mut lines: Vec<String> = report
            .cases
            .iter()
            .filter(|case| !case.ok)
            .map(|case| {
                format!(
                    "FAIL [{}] {} — expected {}, got {}",
                    case.suite, case.scenario, case.expected, case.actual
                )
            })
            .collect();
        lines.push(format!(
            "contract test: {} cases, {} passed, {} failed",
            report.total, report.passed, report.failed
        ));
        lines.join("\n")
    };

    Ok(RunOutput {
        text,
        success: report.failed == 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use causlane_replay::CausalLocation;

    fn explain(accepted: bool, code: Option<&str>) -> ReplayExplain {
        ReplayExplain {
            accepted,
            invariant: None,
            error_code: code.map(ToOwned::to_owned),
            error_detail: None,
            causal_location: CausalLocation::default(),
            checked_invariants: Vec::new(),
            bundle_hash: "b".to_owned(),
            trace_hash: "t".to_owned(),
        }
    }

    fn expectation(result: ExpectedReplayResult, code: Option<&str>) -> ScenarioExpectation {
        ScenarioExpectation {
            scenario_id: "s".to_owned(),
            scenario_hash: "h".to_owned(),
            expected_result: result,
            expected_error_code: code.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn pass_contract_requires_acceptance() {
        let exp = expectation(ExpectedReplayResult::Pass, None);
        assert!(contract_satisfied(&exp, &explain(true, None)));
        assert!(!contract_satisfied(
            &exp,
            &explain(false, Some("ExecutionWithoutBarrier"))
        ));
    }

    #[test]
    fn fail_contract_requires_exact_declared_code() {
        let exp = expectation(ExpectedReplayResult::Fail, Some("ConflictingLeases"));
        // Rejected with the declared code: satisfied.
        assert!(contract_satisfied(
            &exp,
            &explain(false, Some("ConflictingLeases"))
        ));
        // Rejected with a DIFFERENT code: NOT satisfied (the non-vacuity check).
        assert!(!contract_satisfied(
            &exp,
            &explain(false, Some("ExecutionWithoutBarrier"))
        ));
        // Accepted: NOT satisfied.
        assert!(!contract_satisfied(&exp, &explain(true, None)));
    }

    #[test]
    fn fail_without_declared_code_requires_only_rejection() {
        let exp = expectation(ExpectedReplayResult::Fail, None);
        assert!(contract_satisfied(&exp, &explain(false, Some("AnyCode"))));
        assert!(!contract_satisfied(&exp, &explain(true, None)));
    }

    #[test]
    fn tokens_render_outcome_and_expectation() {
        assert_eq!(observed_token(&explain(true, None)), "pass");
        assert_eq!(observed_token(&explain(false, Some("X"))), "fail:X");
        assert_eq!(
            expected_token(&expectation(ExpectedReplayResult::Fail, Some("Y"))),
            "fail:Y"
        );
        assert_eq!(
            expected_token(&expectation(ExpectedReplayResult::Pass, None)),
            "pass"
        );
    }
}
