//! Shared CLI helpers that do not perform platform I/O directly.

use core::fmt;
use std::path::Path;

/// Default root for generated formal artifacts.
pub const DEFAULT_FORMAL_ARTIFACT_DIR: &str = "verification/formal-full";
/// Default root for generated formal receipts.
pub const DEFAULT_FORMAL_RECEIPT_DIR: &str = "verification/formal-full/receipts";
/// Default path for the derived formal coverage report.
pub const DEFAULT_FORMAL_COVERAGE_REPORT: &str = "target/causlane/formal-coverage-report.json";

/// File boundary error with the affected path preserved for CLI rendering.
#[derive(Debug)]
pub enum FileError {
    /// A read operation failed.
    Read {
        /// Path passed to the read adapter.
        path: String,
        /// Adapter error message.
        message: String,
    },
    /// A write or parent-directory creation operation failed.
    Write {
        /// Path passed to the write adapter.
        path: String,
        /// Adapter error message.
        message: String,
    },
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::Read { path, message } => write!(f, "cannot read {path}: {message}"),
            FileError::Write { path, message } => write!(f, "cannot write {path}: {message}"),
        }
    }
}

impl std::error::Error for FileError {}

/// Run a caller-provided read adapter and normalize its error.
#[must_use = "file read errors must be rendered by the CLI boundary"]
pub fn read_file_with<E>(
    path: impl AsRef<Path>,
    read_to_string: impl FnOnce(&Path) -> Result<String, E>,
) -> Result<String, FileError>
where
    E: fmt::Display,
{
    let path = path.as_ref();
    read_to_string(path).map_err(|err| FileError::Read {
        path: path.display().to_string(),
        message: err.to_string(),
    })
}

/// Run caller-provided write adapters after creating the parent directory.
#[must_use = "file write errors must be rendered by the CLI boundary"]
pub fn write_file_with<C, W>(
    path: impl AsRef<Path>,
    content: &str,
    create_dir_all: impl Fn(&Path) -> Result<(), C>,
    write: impl FnOnce(&Path, &str) -> Result<(), W>,
) -> Result<(), FileError>
where
    C: fmt::Display,
    W: fmt::Display,
{
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent).map_err(|err| FileError::Write {
                path: parent.display().to_string(),
                message: err.to_string(),
            })?;
        }
    }
    write(path, content).map_err(|err| FileError::Write {
        path: path.display().to_string(),
        message: err.to_string(),
    })
}

/// Return the value following `flag`, if present.
#[must_use]
pub fn flag_value(args: &[String], flag: &str) -> Option<String> {
    let mut index = 0;
    while let Some(arg) = args.get(index) {
        if arg == flag {
            return args.get(index + 1).cloned();
        }
        index += 1;
    }
    None
}

/// Return whether `flag` is present in argv.
#[must_use]
pub fn flag_present(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

/// Format a checked-at token from a Unix timestamp.
#[must_use]
pub fn checked_at_token_from_unix_secs(seconds: u64) -> String {
    format!("unix:{seconds}")
}

/// Return the deterministic fallback checked-at token.
#[must_use]
pub fn fallback_checked_at_token() -> String {
    "unix:0".to_owned()
}

/// Convert a length to `u32`, saturating at `u32::MAX`.
#[must_use]
pub fn len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// Convert a scenario id into the stable artifact file stem.
#[must_use]
pub fn safe_scenario_stem(raw: &str) -> String {
    let mut name = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            name.push(ch);
        } else {
            name.push('_');
        }
    }
    if name.is_empty() {
        "scenario".to_owned()
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::path::Path;

    use super::{
        checked_at_token_from_unix_secs, fallback_checked_at_token, flag_present, flag_value,
        len_u32, safe_scenario_stem, write_file_with, FileError,
    };

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    #[test]
    fn flags_are_position_independent() {
        let argv = args(&["causlane", "cmd", "--json", "--bundle", "bundle.json"]);
        assert_eq!(
            flag_value(&argv, "--bundle").as_deref(),
            Some("bundle.json")
        );
        assert!(flag_present(&argv, "--json"));
        assert!(!flag_present(&argv, "--missing"));
    }

    #[test]
    fn safe_scenario_stem_is_ascii_and_never_empty() {
        assert_eq!(
            safe_scenario_stem("release-promote.success"),
            "release_promote_success"
        );
        assert_eq!(safe_scenario_stem(""), "scenario");
    }

    #[test]
    fn timestamps_are_stable_tokens() {
        assert_eq!(checked_at_token_from_unix_secs(42), "unix:42");
        assert_eq!(fallback_checked_at_token(), "unix:0");
    }

    #[test]
    fn len_u32_saturates() {
        assert_eq!(len_u32(7), 7);
        assert_eq!(len_u32(usize::MAX), u32::MAX);
    }

    #[test]
    fn write_file_with_creates_parent_directories() -> Result<(), FileError> {
        let created = RefCell::new(Vec::new());
        let written = RefCell::new(Vec::new());
        write_file_with(
            "root/file.txt",
            "payload",
            |path: &Path| {
                created.borrow_mut().push(path.display().to_string());
                Ok::<(), &str>(())
            },
            |path: &Path, content| {
                written
                    .borrow_mut()
                    .push((path.display().to_string(), content.to_owned()));
                Ok::<(), &str>(())
            },
        )?;

        assert_eq!(created.into_inner(), vec!["root"]);
        assert_eq!(
            written.into_inner(),
            vec![("root/file.txt".to_owned(), "payload".to_owned())]
        );
        Ok(())
    }
}
