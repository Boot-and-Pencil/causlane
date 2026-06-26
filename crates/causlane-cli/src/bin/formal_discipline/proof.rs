//! Static proof-lane discipline checks.

use crate::formal_discipline::args::Profile;
use crate::formal_discipline::Findings;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) struct ReceiptResult {
    pub(crate) failed: bool,
    pub(crate) warned: bool,
}

pub(crate) fn check_proof_cheating(profile: Profile) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for path in files_with_ext(Path::new("formal/lean"), "lean") {
        if path
            .components()
            .any(|component| component.as_os_str().to_str() == Some(".lake"))
        {
            continue;
        }
        scan_file_for_tokens(&path, CommentKind::Lean, &lean_tokens(), &mut errors);
    }
    for path in files_with_ext(Path::new("formal/lean4/generated"), "lean") {
        scan_file_for_tokens(&path, CommentKind::Lean, &lean_tokens(), &mut errors);
    }
    if profile.is_proof() {
        for path in files_with_ext(Path::new("formal/verus"), "rs") {
            scan_file_for_tokens(&path, CommentKind::Rust, &verus_tokens(), &mut errors);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(crate) fn check_generated_artifacts() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (dir, extension) in [
        ("formal/alloy/generated", "als"),
        ("formal/p/generated", "p"),
        ("formal/kani/generated", "rs"),
        ("formal/verus/generated", "rs"),
        ("formal/lean4/generated", "lean"),
    ] {
        for path in files_with_ext(Path::new(dir), extension) {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    for field in [
                        "source_bundle_hash",
                        "formal_ir_hash",
                        "scenario_hash",
                        "generator_version",
                        "invariant_ids",
                    ] {
                        if !content.contains(field) {
                            errors.push(format!(
                                "{} missing generated artifact header field {field}",
                                path.display()
                            ));
                        }
                    }
                }
                Err(err) => errors.push(format!("{} cannot be read: {err}", path.display())),
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub(crate) fn check_receipts(profile: Profile, findings: &mut Findings) -> ReceiptResult {
    let mut result = ReceiptResult {
        failed: false,
        warned: false,
    };
    for path in direct_receipt_files(Path::new("formal/receipts")) {
        check_receipt_file(profile, &path, findings, &mut result);
    }
    result
}

fn check_receipt_file(
    profile: Profile,
    path: &Path,
    findings: &mut Findings,
    result: &mut ReceiptResult,
) {
    let label = path.display().to_string();
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            result.failed = true;
            findings
                .errors
                .push(format!("{label} cannot be read: {err}"));
            return;
        }
    };
    let json: Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(err) => {
            result.failed = true;
            findings
                .errors
                .push(format!("{label} JSON parse failed: {err}"));
            return;
        }
    };
    let actual_result = json.get("actual_result").and_then(Value::as_str);
    let exit_code = json.get("exit_code").and_then(Value::as_i64);
    let has_exit_code = json.get("exit_code").is_some();
    match actual_result {
        None => {
            result.failed = true;
            findings
                .errors
                .push(format!("{label} missing actual_result"));
        }
        Some("pass") => {
            if exit_code != Some(0) {
                result.failed = true;
                findings
                    .errors
                    .push(format!("{label} records pass without exit_code 0"));
            }
        }
        Some("non_blocking_skipped") => {
            if profile.is_proof() {
                result.failed = true;
                findings.errors.push(format!(
                    "{label} records non_blocking_skipped in proof/all profile"
                ));
            } else if exit_code == Some(0) {
                result.warned = true;
                findings.warnings.push(format!(
                    "{label} records policy-allowed non_blocking_skipped; not counted as pass"
                ));
            } else {
                result.failed = true;
                findings.errors.push(format!(
                    "{label} records non_blocking_skipped without exit_code 0"
                ));
            }
        }
        Some(other) => {
            if !has_exit_code {
                result.failed = true;
                findings
                    .errors
                    .push(format!("{label} missing exit_code for result {other}"));
            } else if exit_code.is_none() {
                result.warned = true;
                findings.warnings.push(format!(
                    "{label} is pending with result {other} and null exit_code"
                ));
            }
        }
    }
}

fn scan_file_for_tokens(
    path: &Path,
    comment_kind: CommentKind,
    tokens: &[ForbiddenToken],
    errors: &mut Vec<String>,
) {
    let Ok(content) = std::fs::read_to_string(path) else {
        errors.push(format!("{} cannot be read", path.display()));
        return;
    };
    for (line_number, line) in content.lines().enumerate() {
        let code = strip_line_comment(line, comment_kind);
        for token in tokens {
            if token.matches(code) {
                errors.push(format!(
                    "{}:{} contains forbidden proof token {}",
                    path.display(),
                    line_number.saturating_add(1),
                    token.label
                ));
            }
        }
    }
}

fn direct_receipt_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".tool-run.json"))
        {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn files_with_ext(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_with_ext(dir, extension, &mut files);
    files.sort();
    files
}

fn collect_files_with_ext(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_ext(&path, extension, files);
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value == extension)
        {
            files.push(path);
        }
    }
}

#[derive(Clone, Copy)]
enum CommentKind {
    Lean,
    Rust,
}

fn strip_line_comment(line: &str, kind: CommentKind) -> &str {
    let marker = match kind {
        CommentKind::Lean => "--",
        CommentKind::Rust => "//",
    };
    line.split_once(marker)
        .map_or(line, |(code, _comment)| code)
}

struct ForbiddenToken {
    label: &'static str,
    needle: &'static str,
    mode: TokenMode,
}

impl ForbiddenToken {
    fn matches(&self, code: &str) -> bool {
        match self.mode {
            TokenMode::Word => contains_word(code, self.needle),
            TokenMode::Text => code.contains(self.needle),
            TokenMode::AssumeCall => contains_assume_call(code),
        }
    }
}

#[derive(Clone, Copy)]
enum TokenMode {
    Word,
    Text,
    AssumeCall,
}

fn lean_tokens() -> [ForbiddenToken; 3] {
    [
        ForbiddenToken {
            label: "sorry",
            needle: "sorry",
            mode: TokenMode::Word,
        },
        ForbiddenToken {
            label: "admit",
            needle: "admit",
            mode: TokenMode::Word,
        },
        ForbiddenToken {
            label: "axiom",
            needle: "axiom",
            mode: TokenMode::Word,
        },
    ]
}

fn verus_tokens() -> [ForbiddenToken; 5] {
    [
        ForbiddenToken {
            label: "assume",
            needle: "assume",
            mode: TokenMode::AssumeCall,
        },
        ForbiddenToken {
            label: "#[verifier::external_body]",
            needle: "#[verifier::external_body]",
            mode: TokenMode::Text,
        },
        ForbiddenToken {
            label: "unimplemented!",
            needle: "unimplemented!",
            mode: TokenMode::Text,
        },
        ForbiddenToken {
            label: "todo!",
            needle: "todo!",
            mode: TokenMode::Text,
        },
        ForbiddenToken {
            label: "admit",
            needle: "admit",
            mode: TokenMode::Word,
        },
    ]
}

fn contains_assume_call(code: &str) -> bool {
    code.match_indices("assume").any(|(position, _)| {
        let Some((before, after_with_word)) = code.get(..position).zip(code.get(position..)) else {
            return false;
        };
        let after = after_with_word.strip_prefix("assume").unwrap_or_default();
        let boundary_before = before
            .chars()
            .next_back()
            .is_none_or(|ch| !is_ident_char(ch));
        let boundary_after = after
            .chars()
            .next()
            .is_none_or(|ch| ch.is_whitespace() || ch == '(');
        if !boundary_before || !boundary_after {
            return false;
        }
        after.chars().find(|ch| !ch.is_whitespace()) == Some('(')
    })
}

fn contains_word(code: &str, word: &str) -> bool {
    code.match_indices(word).any(|(position, _)| {
        let Some((before, after_with_word)) = code.get(..position).zip(code.get(position..)) else {
            return false;
        };
        let Some(after) = after_with_word.strip_prefix(word) else {
            return false;
        };
        before
            .chars()
            .next_back()
            .is_none_or(|ch| !is_ident_char(ch))
            && after.chars().next().is_none_or(|ch| !is_ident_char(ch))
    })
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_discipline_lean_comment_tokens_do_not_match() {
        let code = strip_line_comment("theorem ok := by trivial -- sorry axiom", CommentKind::Lean);
        assert!(!lean_tokens().iter().any(|token| token.matches(code)));
        assert!(lean_tokens()
            .iter()
            .any(|token| token.matches("theorem bad := by sorry")));
    }

    #[test]
    fn formal_discipline_verus_assume_call_matches() {
        assert!(contains_assume_call("proof { assume(false); }"));
        assert!(contains_assume_call("proof { assume (false); }"));
        assert!(!contains_assume_call("let assumed = true;"));
    }

    #[test]
    fn formal_discipline_receipt_pass_requires_zero_exit() {
        let mut findings = Findings::default();
        let mut result = ReceiptResult {
            failed: false,
            warned: false,
        };
        let path = temp_receipt("pass_nonzero", r#"{"actual_result":"pass","exit_code":1}"#);
        check_receipt_file(Profile::Rust, &path, &mut findings, &mut result);
        let _ignored = std::fs::remove_file(path);
        assert!(result.failed);
        assert!(findings
            .errors
            .iter()
            .any(|error| error.contains("exit_code 0")));
    }

    #[test]
    fn formal_discipline_receipt_pending_null_warns() {
        let mut findings = Findings::default();
        let mut result = ReceiptResult {
            failed: false,
            warned: false,
        };
        let path = temp_receipt(
            "pending_null",
            r#"{"actual_result":"not_run","exit_code":null}"#,
        );
        check_receipt_file(Profile::Rust, &path, &mut findings, &mut result);
        let _ignored = std::fs::remove_file(path);
        assert!(!result.failed);
        assert!(result.warned);
    }

    #[test]
    fn formal_discipline_non_blocking_skip_is_profile_aware() {
        let path = temp_receipt(
            "non_blocking_skipped",
            r#"{"actual_result":"non_blocking_skipped","exit_code":0}"#,
        );

        let mut rust_findings = Findings::default();
        let mut rust_result = ReceiptResult {
            failed: false,
            warned: false,
        };
        check_receipt_file(Profile::Rust, &path, &mut rust_findings, &mut rust_result);
        assert!(!rust_result.failed);
        assert!(rust_result.warned);

        let mut proof_findings = Findings::default();
        let mut proof_result = ReceiptResult {
            failed: false,
            warned: false,
        };
        check_receipt_file(
            Profile::Proof,
            &path,
            &mut proof_findings,
            &mut proof_result,
        );
        let _ignored = std::fs::remove_file(path);
        assert!(proof_result.failed);
        assert!(proof_findings
            .errors
            .iter()
            .any(|error| error.contains("proof/all profile")));
    }

    fn temp_receipt(label: &str, content: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let suffix = std::process::id();
        path.push(format!("causlane_formal_discipline_{label}_{suffix}.json"));
        assert!(std::fs::write(&path, content).is_ok());
        path
    }
}
