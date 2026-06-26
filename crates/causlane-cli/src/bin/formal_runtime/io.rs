//! Shared formal CLI runtime I/O adapter.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use causlane_cli::app::formal::{FormalIo, FormalIoError};
use causlane_cli::cli_shared::{checked_at_token_from_unix_secs, fallback_checked_at_token};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct StdFormalIo;

impl FormalIo for StdFormalIo {
    fn read_to_string(&mut self, path: &Path) -> Result<String, FormalIoError> {
        std::fs::read_to_string(path).map_err(|err| FormalIoError::new(err.to_string()))
    }

    fn create_dir_all(&mut self, path: &Path) -> Result<(), FormalIoError> {
        std::fs::create_dir_all(path).map_err(|err| FormalIoError::new(err.to_string()))
    }

    fn write_string(&mut self, path: &Path, content: &str) -> Result<(), FormalIoError> {
        std::fs::write(path, content).map_err(|err| FormalIoError::new(err.to_string()))
    }

    fn read_dir_paths(&mut self, path: &Path) -> Result<Option<Vec<PathBuf>>, FormalIoError> {
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(FormalIoError::new(err.to_string())),
        };
        let mut paths = Vec::new();
        for entry in entries {
            paths.push(
                entry
                    .map_err(|err| FormalIoError::new(err.to_string()))?
                    .path(),
            );
        }
        paths.sort();
        Ok(Some(paths))
    }

    fn checked_at_token(&self) -> String {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => checked_at_token_from_unix_secs(duration.as_secs()),
            Err(_err) => fallback_checked_at_token(),
        }
    }
}
