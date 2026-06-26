//! Path and diff classification for the formal discipline checker.

use crate::formal_discipline::args::{CliError, DiffSource};

pub(crate) fn changed_files(source: &DiffSource) -> Result<Vec<String>, CliError> {
    match source {
        DiffSource::NoDiff => Ok(Vec::new()),
        DiffSource::ChangedFiles(path) => {
            let content = std::fs::read_to_string(path).map_err(|err| CliError::Io {
                path: path.clone(),
                message: err.to_string(),
            })?;
            Ok(content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(normalize_path)
                .collect())
        }
    }
}

pub(crate) fn protocol_critical_files(files: &[String]) -> Vec<String> {
    files
        .iter()
        .filter(|path| is_protocol_critical(path))
        .cloned()
        .collect()
}

pub(crate) fn impact_record_found(files: &[String]) -> bool {
    files.iter().any(|path| {
        let normalized = normalize_path(path);
        if !is_impact_record_candidate(&normalized) {
            return false;
        }
        normalized.contains("formal-impact")
            || std::fs::read_to_string(&normalized).is_ok_and(|content| {
                content.contains("Formal Impact Record") || content.contains("formal impact")
            })
    })
}

pub(crate) fn is_protocol_critical(path: &str) -> bool {
    let path = normalize_path(path);
    if is_generated_exemption(&path) {
        return false;
    }
    path.starts_with("contracts/examples/")
        || path.starts_with("contracts/scenarios/")
        || path.starts_with("contracts/schema/")
        || path.starts_with("crates/causlane-contracts/src/")
        || path.starts_with("crates/causlane-core/src/domain/")
        || path.starts_with("crates/causlane-replay/src/")
        || path.starts_with("crates/causlane-codegen/src/")
        || path == "crates/causlane-runtime/src/guarded_executor.rs"
        || path == "crates/causlane-runtime/src/authz.rs"
        || path == "crates/causlane-cli/src/bin/causlane-formal.rs"
        || path == "crates/causlane-cli/src/bin/causlane-formal-discipline.rs"
        || path.starts_with("crates/causlane-cli/src/bin/formal_discipline/")
        || is_cli_formal_source(&path)
        || is_formal_tool(&path)
        || path.starts_with("docs/invariants/")
        || path.starts_with("docs/formal-exceptions.")
        || path.starts_with("formal/")
}

// Source/receipt extensions are conventionally lower-case; case-sensitive
// extension matching is intentional here.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn is_generated_exemption(path: &str) -> bool {
    (path.starts_with("formal/") && path.contains("/generated/"))
        || (path.starts_with("formal/receipts/")
            && path.ends_with(".json")
            && path.matches('/').count() == 2)
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn is_cli_formal_source(path: &str) -> bool {
    path.starts_with("crates/causlane-cli/src/formal_") && path.ends_with(".rs")
}

fn is_formal_tool(path: &str) -> bool {
    path.starts_with("tools/formal-")
        || path == "tools/coverage-matrix"
        || path == "tools/proof-refinement-scope"
        || path == "tools/doc_projection.py"
}

fn is_impact_record_candidate(path: &str) -> bool {
    path.starts_with("docs/templates/")
        || path.starts_with("docs/formal/")
        || path.starts_with("docs/product-track/")
        || path.starts_with("docs/adr/")
}

fn normalize_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_discipline_marks_protocol_paths_critical() {
        assert!(is_protocol_critical("crates/causlane-replay/src/lib.rs"));
        assert!(is_protocol_critical(
            "crates/causlane-cli/src/bin/formal_discipline/paths.rs"
        ));
        assert!(is_protocol_critical("tools/formal-discipline-check"));
        assert!(is_protocol_critical("tools/proof-refinement-scope"));
        assert!(is_protocol_critical("tools/doc_projection.py"));
        assert!(!is_protocol_critical("docs/README.md"));
    }

    #[test]
    fn formal_discipline_exempts_generated_outputs() {
        assert!(!is_protocol_critical(
            "formal/lean4/generated/release_promote_success.lean"
        ));
        assert!(!is_protocol_critical(
            "formal/receipts/release_promote_success.lean4.tool-run.json"
        ));
        assert!(is_protocol_critical(
            "formal/receipts/examples/alloy-core.sample.json"
        ));
    }

    #[test]
    fn formal_discipline_detects_impact_record_path() {
        assert!(impact_record_found(&[String::from(
            "docs/templates/formal-impact-record.md"
        )]));
    }
}
