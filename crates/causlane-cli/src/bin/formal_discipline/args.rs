//! Argument parsing for the formal discipline checker.

use core::fmt;

pub(crate) const DEFAULT_MANIFEST: &str =
    "verification/formal-full/obligations/lifecycle_product_obligations.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Profile {
    Base,
    Rust,
    Ci,
    Proof,
    All,
}

impl Profile {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "base" => Some(Self::Base),
            "rust" => Some(Self::Rust),
            "ci" => Some(Self::Ci),
            "proof" => Some(Self::Proof),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Rust => "rust",
            Self::Ci => "ci",
            Self::Proof => "proof",
            Self::All => "all",
        }
    }

    pub(crate) fn is_proof(self) -> bool {
        matches!(self, Self::Proof | Self::All)
    }
}

#[derive(Debug)]
pub(crate) enum DiffSource {
    ChangedFiles(String),
    NoDiff,
}

impl DiffSource {
    pub(crate) fn is_no_diff(&self) -> bool {
        matches!(self, Self::NoDiff)
    }
}

#[derive(Debug)]
pub(crate) struct Args {
    pub(crate) profile: Profile,
    pub(crate) diff_source: DiffSource,
    pub(crate) manifest: String,
    pub(crate) json: bool,
}

#[derive(Debug)]
pub(crate) enum CliError {
    Usage(String),
    Io { path: String, message: String },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::Io { path, message } => write!(f, "cannot read {path}: {message}"),
        }
    }
}

pub(crate) fn parse_args(argv: &[String]) -> Result<Args, CliError> {
    if argv.iter().any(|arg| arg == "-h" || arg == "--help") {
        return Err(CliError::Usage(usage()));
    }

    let mut profile = None;
    let mut changed_files = None;
    let mut no_diff = false;
    let mut manifest = DEFAULT_MANIFEST.to_owned();
    let mut json = false;
    let mut index = 1;
    while let Some(arg) = argv.get(index) {
        match arg.as_str() {
            "--profile" => {
                index += 1;
                let value = argv
                    .get(index)
                    .ok_or_else(|| CliError::Usage("--profile requires a value".to_owned()))?;
                profile = Profile::parse(value);
                if profile.is_none() {
                    return Err(CliError::Usage(format!(
                        "unsupported --profile {value}; expected base|rust|ci|proof|all"
                    )));
                }
            }
            "--changed-files" => {
                index += 1;
                changed_files = Some(
                    argv.get(index)
                        .ok_or_else(|| {
                            CliError::Usage("--changed-files requires a value".to_owned())
                        })?
                        .clone(),
                );
            }
            "--from-git" => {
                return Err(CliError::Usage(
                    "--from-git is handled by tools/formal-discipline-check wrapper".to_owned(),
                ));
            }
            "--no-diff" => no_diff = true,
            "--manifest" => {
                index += 1;
                manifest
                    .clone_from(argv.get(index).ok_or_else(|| {
                        CliError::Usage("--manifest requires a value".to_owned())
                    })?);
            }
            "--json" => json = true,
            other => {
                return Err(CliError::Usage(format!(
                    "unknown argument {other}\n{}",
                    usage()
                )));
            }
        }
        index += 1;
    }

    let profile = profile.ok_or_else(|| CliError::Usage("--profile is required".to_owned()))?;
    let source_count = usize::from(changed_files.is_some()) + usize::from(no_diff);
    if source_count != 1 {
        return Err(CliError::Usage(
            "provide exactly one of --changed-files or --no-diff".to_owned(),
        ));
    }
    let diff_source = changed_files.map_or(DiffSource::NoDiff, DiffSource::ChangedFiles);
    Ok(Args {
        profile,
        diff_source,
        manifest,
        json,
    })
}

fn usage() -> String {
    [
        "usage:",
        "  tools/formal-discipline-check --profile base|rust|ci|proof|all \\",
        "    [--changed-files path.txt | --from-git base...head | --no-diff] \\",
        "    [--manifest verification/formal-full/obligations/lifecycle_product_obligations.yaml] [--json]",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum TestError {
        Cli,
        WrongProfile,
        WrongDiffSource,
        WrongManifest,
        WrongJson,
        ExpectedUsage,
        WrongProofProfile,
    }

    impl From<CliError> for TestError {
        fn from(_err: CliError) -> Self {
            Self::Cli
        }
    }

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn parse_changed_files_with_manifest_and_json() -> Result<(), TestError> {
        let parsed = parse_args(&argv(&[
            "formal-discipline-check",
            "--profile",
            "ci",
            "--changed-files",
            "changed.txt",
            "--manifest",
            "custom.yaml",
            "--json",
        ]))?;
        if parsed.profile != Profile::Ci {
            return Err(TestError::WrongProfile);
        }
        match parsed.diff_source {
            DiffSource::ChangedFiles(path) if path == "changed.txt" => {}
            DiffSource::ChangedFiles(_) | DiffSource::NoDiff => {
                return Err(TestError::WrongDiffSource);
            }
        }
        if parsed.manifest != "custom.yaml" {
            return Err(TestError::WrongManifest);
        }
        if !parsed.json {
            return Err(TestError::WrongJson);
        }
        Ok(())
    }

    #[test]
    fn parse_no_diff_and_reject_invalid_combinations() -> Result<(), TestError> {
        let parsed = parse_args(&argv(&[
            "formal-discipline-check",
            "--profile",
            "proof",
            "--no-diff",
        ]))?;
        if !parsed.diff_source.is_no_diff() {
            return Err(TestError::WrongDiffSource);
        }
        if !parsed.profile.is_proof() || parsed.profile.as_str() != "proof" {
            return Err(TestError::WrongProofProfile);
        }
        for invalid in [
            argv(&["formal-discipline-check", "--profile", "ci"]),
            argv(&[
                "formal-discipline-check",
                "--profile",
                "ci",
                "--changed-files",
                "changed.txt",
                "--no-diff",
            ]),
            argv(&[
                "formal-discipline-check",
                "--profile",
                "unknown",
                "--no-diff",
            ]),
            argv(&["formal-discipline-check", "--from-git", "main...HEAD"]),
        ] {
            if !matches!(parse_args(&invalid), Err(CliError::Usage(_))) {
                return Err(TestError::ExpectedUsage);
            }
        }
        Ok(())
    }
}
