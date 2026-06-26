use core::fmt;

/// Errors raised by formal artifact generation and stale checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenError {
    /// A generated artifact header is missing or malformed.
    Header(String),
    /// Scenario facts could not be generated.
    Scenario(String),
    /// The generated artifact no longer matches its source bundle.
    Stale(String),
    /// A receipt JSON document could not be decoded.
    Receipt(String),
    /// Two distinct domain names map to the same generated target identifier,
    /// which would silently merge them in the formal model.
    Collision(String),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::Header(msg) => write!(f, "generated header error: {msg}"),
            CodegenError::Scenario(msg) => write!(f, "scenario codegen error: {msg}"),
            CodegenError::Stale(msg) => write!(f, "stale generated artifact: {msg}"),
            CodegenError::Receipt(msg) => write!(f, "receipt error: {msg}"),
            CodegenError::Collision(msg) => write!(f, "identifier collision: {msg}"),
        }
    }
}

impl std::error::Error for CodegenError {}
