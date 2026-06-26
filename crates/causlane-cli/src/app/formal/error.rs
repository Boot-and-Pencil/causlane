//! Error surface for the shared formal orchestration service.

use core::fmt;

use causlane_codegen::CodegenError;
use causlane_contracts::ContractError;
use causlane_replay::ReplayError;

use crate::cli_shared::FileError;

/// Errors raised by the shared formal service.
#[derive(Debug)]
pub enum FormalServiceError {
    /// File read/write adapter failure.
    File(FileError),
    /// Directory traversal or other non-file adapter failure.
    Io {
        /// Path passed to the adapter.
        path: String,
        /// Adapter error message.
        message: String,
    },
    /// Contract/bundle parsing or validation failure.
    Contract(ContractError),
    /// Replay/scenario parsing or verification failure.
    Replay(ReplayError),
    /// Formal codegen, receipt or coverage failure.
    Codegen(CodegenError),
    /// CLI usage error delegated through the service.
    Usage(String),
}

impl fmt::Display for FormalServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormalServiceError::File(err) => write!(f, "{err}"),
            FormalServiceError::Io { path, message } => write!(f, "cannot read {path}: {message}"),
            FormalServiceError::Contract(err) => write!(f, "{err}"),
            FormalServiceError::Replay(err) => write!(f, "{err}"),
            FormalServiceError::Codegen(err) => write!(f, "{err}"),
            FormalServiceError::Usage(text) => write!(f, "{text}"),
        }
    }
}

impl From<FileError> for FormalServiceError {
    fn from(err: FileError) -> Self {
        FormalServiceError::File(err)
    }
}

impl From<ContractError> for FormalServiceError {
    fn from(err: ContractError) -> Self {
        FormalServiceError::Contract(err)
    }
}

impl From<ReplayError> for FormalServiceError {
    fn from(err: ReplayError) -> Self {
        FormalServiceError::Replay(err)
    }
}

impl From<CodegenError> for FormalServiceError {
    fn from(err: CodegenError) -> Self {
        FormalServiceError::Codegen(err)
    }
}
