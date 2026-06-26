//! Formal discipline checker implementation.

use crate::formal_discipline::args::{Args, CliError, DiffSource};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

mod adequacy;
mod args;
mod manifest;
mod paths;
mod proof;

#[derive(Default)]
pub(crate) struct Findings {
    pub(crate) errors: Vec<String>,
    pub(crate) warnings: Vec<String>,
}

struct CheckResults {
    args: Args,
    changed_files: Vec<String>,
    protocol_critical_files: Vec<String>,
    impact_record_found: bool,
    manifest_status: String,
    exceptions_status: String,
    coverage_drift_status: String,
    adequacy_status: String,
    proof_cheating_status: String,
    artifact_status: String,
    receipt_status: String,
    findings: Findings,
}

impl CheckResults {
    fn status(&self) -> &'static str {
        if self.findings.errors.is_empty() {
            "pass"
        } else {
            "fail"
        }
    }
}

#[derive(Debug)]
enum CheckError {
    Message(String),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

pub(crate) fn run_cli(argv: &[String]) -> ExitCode {
    match args::parse_args(argv) {
        Ok(parsed) => {
            let results = run_checks(parsed);
            emit_results(&results);
            if results.findings.errors.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(CliError::Usage(message)) => {
            write_stderr(&format!("{message}\n"));
            ExitCode::from(2)
        }
        Err(CliError::Io { path, message }) => {
            write_stderr(&format!("cannot read {path}: {message}\n"));
            ExitCode::from(1)
        }
    }
}

fn run_checks(args: Args) -> CheckResults {
    let mut findings = Findings::default();
    if args.diff_source.is_no_diff() && ci_env() {
        findings
            .errors
            .push("--no-diff is forbidden when CI/GITHUB_ACTIONS is set".to_owned());
    }

    let changed_files = match paths::changed_files(&args.diff_source) {
        Ok(files) => files,
        Err(err) => {
            findings.errors.push(err.to_string());
            Vec::new()
        }
    };
    let protocol_critical_files = paths::protocol_critical_files(&changed_files);
    let impact_record_found = paths::impact_record_found(&changed_files);
    if !protocol_critical_files.is_empty() && !impact_record_found {
        findings.errors.push(format!(
            "protocol-critical changes require a Formal Impact Record: {}",
            protocol_critical_files.join(", ")
        ));
    }

    let manifest_status = status_from_result(
        manifest::validate_manifest(&args.manifest),
        &mut findings,
        "manifest",
    );
    let exceptions_status = validate_exceptions(&mut findings);
    let coverage_drift_status = validate_coverage_drift(&args.diff_source, &mut findings);
    let adequacy_status = adequacy::check(&args.manifest, &args.diff_source, &mut findings);
    let proof_cheating_status = status_from_result(
        proof::check_proof_cheating(args.profile),
        &mut findings,
        "proof cheating",
    );
    let artifact_status = status_from_result(
        proof::check_generated_artifacts(),
        &mut findings,
        "generated artifacts",
    );
    let receipt_result = proof::check_receipts(args.profile, &mut findings);
    let receipt_status = if receipt_result.failed {
        "fail".to_owned()
    } else if receipt_result.warned {
        "warn".to_owned()
    } else {
        "pass".to_owned()
    };

    CheckResults {
        args,
        changed_files,
        protocol_critical_files,
        impact_record_found,
        manifest_status,
        exceptions_status,
        coverage_drift_status,
        adequacy_status,
        proof_cheating_status,
        artifact_status,
        receipt_status,
        findings,
    }
}

fn status_from_result(
    result: Result<(), Vec<String>>,
    findings: &mut Findings,
    label: &str,
) -> String {
    match result {
        Ok(()) => "pass".to_owned(),
        Err(errors) => {
            findings
                .errors
                .extend(errors.into_iter().map(|error| format!("{label}: {error}")));
            "fail".to_owned()
        }
    }
}

fn validate_exceptions(findings: &mut Findings) -> String {
    let path = "docs/formal-exceptions.json";
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            findings
                .errors
                .push(format!("{path} cannot be read: {err}"));
            return "fail".to_owned();
        }
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(err) => {
            findings
                .errors
                .push(format!("{path} JSON parse failed: {err}"));
            return "fail".to_owned();
        }
    };
    let mut failed = false;
    if json.get("schema_version").and_then(Value::as_i64) != Some(1) {
        failed = true;
        findings
            .errors
            .push("formal exceptions schema_version must be 1".to_owned());
    }
    let today = current_utc_date();
    let exceptions = json.get("exceptions").and_then(Value::as_array);
    let Some(exceptions) = exceptions else {
        findings
            .errors
            .push("formal exceptions must contain exceptions array".to_owned());
        return "fail".to_owned();
    };
    for exception in exceptions {
        let id = exception
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        match exception.get("allowed_until").and_then(Value::as_str) {
            Some(date) if valid_date(date) && date >= today.as_str() => {}
            Some(date) if valid_date(date) => {
                failed = true;
                findings.errors.push(format!(
                    "formal exception {id} expired on {date}; today is {today}"
                ));
            }
            Some(date) => {
                failed = true;
                findings.errors.push(format!(
                    "formal exception {id} has invalid allowed_until {date}"
                ));
            }
            None => {
                failed = true;
                findings
                    .errors
                    .push(format!("formal exception {id} missing allowed_until"));
            }
        }
    }
    if failed {
        "fail".to_owned()
    } else {
        "pass".to_owned()
    }
}

fn validate_coverage_drift(diff_source: &DiffSource, findings: &mut Findings) -> String {
    let report_path = "target/causlane/formal-coverage-report.json";
    if !std::path::Path::new(report_path).exists() {
        findings.warnings.push(format!(
            "{report_path} is absent; coverage drift check skipped"
        ));
        return "skipped".to_owned();
    }
    let report = match read_json(report_path) {
        Ok(value) => value,
        Err(err) => {
            findings.errors.push(err.to_string());
            return "fail".to_owned();
        }
    };
    let doc_path = "docs/invariants/coverage-matrix.json";
    let docs = match read_json(doc_path) {
        Ok(value) => value,
        Err(err) => {
            findings.errors.push(err.to_string());
            return "fail".to_owned();
        }
    };
    let report_map = match report_coverage_map(&report) {
        Ok(map) => map,
        Err(err) => {
            findings.errors.push(err.to_string());
            return "fail".to_owned();
        }
    };
    let docs_map = match docs_coverage_map(&docs) {
        Ok(map) => map,
        Err(err) => {
            findings.errors.push(err.to_string());
            return "fail".to_owned();
        }
    };
    if report_map == docs_map {
        return "pass".to_owned();
    }

    let message = "coverage matrix drift detected between target/causlane/formal-coverage-report.json and docs/invariants/coverage-matrix.json";
    if diff_source.is_no_diff() {
        findings.warnings.push(format!(
            "{message}; local --no-diff run reports this as warning"
        ));
        "warn".to_owned()
    } else {
        findings.errors.push(message.to_owned());
        "fail".to_owned()
    }
}

fn read_json(path: &str) -> Result<Value, CheckError> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| CheckError::Message(format!("{path} cannot be read: {err}")))?;
    serde_json::from_str(&content)
        .map_err(|err| CheckError::Message(format!("{path} JSON parse failed: {err}")))
}

fn report_coverage_map(value: &Value) -> Result<BTreeMap<String, Value>, CheckError> {
    let Some(entries) = value.get("invariant_coverage").and_then(Value::as_array) else {
        return Err(CheckError::Message(
            "coverage report missing invariant_coverage array".to_owned(),
        ));
    };
    let mut map = BTreeMap::new();
    for entry in entries {
        let Some(id) = entry.get("invariant_id").and_then(Value::as_str) else {
            return Err(CheckError::Message(
                "coverage report entry missing invariant_id".to_owned(),
            ));
        };
        map.insert(id.to_owned(), canonical_coverage_entry(entry, false));
    }
    Ok(map)
}

fn docs_coverage_map(value: &Value) -> Result<BTreeMap<String, Value>, CheckError> {
    let Some(entries) = value.get("invariants").and_then(Value::as_array) else {
        return Err(CheckError::Message(
            "coverage matrix missing invariants array".to_owned(),
        ));
    };
    let mut map = BTreeMap::new();
    for entry in entries {
        let Some(id) = entry.get("id").and_then(Value::as_str) else {
            return Err(CheckError::Message(
                "coverage matrix entry missing id".to_owned(),
            ));
        };
        map.insert(id.to_owned(), canonical_coverage_entry(entry, true));
    }
    Ok(map)
}

fn canonical_coverage_entry(entry: &Value, lanes_nested: bool) -> Value {
    let lanes = if lanes_nested {
        entry.get("lanes").unwrap_or(&Value::Null)
    } else {
        entry
    };
    json!({
        "replay": lanes.get("replay").cloned().unwrap_or(Value::Null),
        "alloy": lanes.get("alloy").cloned().unwrap_or(Value::Null),
        "p": lanes.get("p").cloned().unwrap_or(Value::Null),
        "kani": lanes.get("kani").cloned().unwrap_or(Value::Null),
        "verus": lanes.get("verus").cloned().unwrap_or(Value::Null),
        "lean4": lanes.get("lean4").cloned().unwrap_or(Value::Null),
        "status": entry.get("status").cloned().unwrap_or(Value::Null),
        "check_ids": entry.get("check_ids").cloned().unwrap_or(Value::Null),
    })
}

fn emit_results(results: &CheckResults) {
    if results.args.json {
        let output = json!({
            "schema_version": 1,
            "status": results.status(),
            "profile": results.args.profile.as_str(),
            "protocol_critical_change": !results.protocol_critical_files.is_empty(),
            "impact_record_found": results.impact_record_found,
            "manifest_status": results.manifest_status,
            "exceptions_status": results.exceptions_status,
            "coverage_drift_status": results.coverage_drift_status,
            "adequacy_status": results.adequacy_status,
            "proof_cheating_status": results.proof_cheating_status,
            "artifact_status": results.artifact_status,
            "receipt_status": results.receipt_status,
            "changed_files": results.changed_files,
            "protocol_critical_files": results.protocol_critical_files,
            "errors": results.findings.errors,
            "warnings": results.findings.warnings,
        });
        write_stdout(&format!("{output}\n"));
    } else {
        write_stdout(&format!("formal-discipline-check: {}\n", results.status()));
        for error in &results.findings.errors {
            write_stderr(&format!("error: {error}\n"));
        }
        for warning in &results.findings.warnings {
            write_stderr(&format!("warning: {warning}\n"));
        }
    }
}

fn ci_env() -> bool {
    env_true("CI") || env_true("GITHUB_ACTIONS")
}

fn env_true(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE"))
}

fn current_utc_date() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_param = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_param + 2) / 5 + 1;
    let month = month_param + if month_param < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (
        year,
        u32::try_from(month).unwrap_or(1),
        u32::try_from(day).unwrap_or(1),
    )
}

fn valid_date(value: &str) -> bool {
    let mut parts = value.split('-');
    let year = parts.next();
    let month = parts.next();
    let day = parts.next();
    if parts.next().is_some() {
        return false;
    }
    year.is_some_and(|part| part.len() == 4 && part.chars().all(|ch| ch.is_ascii_digit()))
        && month.is_some_and(|part| {
            part.len() == 2
                && part.chars().all(|ch| ch.is_ascii_digit())
                && ("01"..="12").contains(&part)
        })
        && day.is_some_and(|part| {
            part.len() == 2
                && part.chars().all(|ch| ch.is_ascii_digit())
                && ("01"..="31").contains(&part)
        })
}

fn write_stdout(message: &str) {
    let _ignored = std::io::stdout().write_all(message.as_bytes());
}

fn write_stderr(message: &str) {
    let _ignored = std::io::stderr().write_all(message.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_discipline_civil_date_matches_unix_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_892), (2027, 3, 15));
    }

    #[test]
    fn formal_discipline_date_validation_is_strict_shape() {
        assert!(valid_date("2026-09-01"));
        assert!(!valid_date("2026-9-1"));
        assert!(!valid_date("2026-13-01"));
    }

    #[test]
    fn formal_discipline_no_diff_local_allows_stale_coverage_warning() {
        let mut findings = Findings::default();
        let status = validate_coverage_drift(&DiffSource::NoDiff, &mut findings);
        assert!(matches!(status.as_str(), "pass" | "warn" | "skipped"));
    }
}
