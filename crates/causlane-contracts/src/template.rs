//! Subject/circumstance template resolution shared by bundle validation,
//! replay and generated formal facts.

use core::fmt;

use serde::{Deserialize, Serialize};

const SUBJECT_NAMESPACE: &str = "subject";
const CIRCUMSTANCE_NAMESPACE: &str = "circumstance";

/// Namespace of a template binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateNamespace {
    /// Subject field namespace.
    Subject,
    /// Circumstance field namespace.
    Circumstance,
}

impl TemplateNamespace {
    fn from_token(token: &str) -> Result<Self, TemplateError> {
        if token == SUBJECT_NAMESPACE {
            return Ok(Self::Subject);
        }
        if token == CIRCUMSTANCE_NAMESPACE {
            return Ok(Self::Circumstance);
        }
        Err(TemplateError::UnsupportedNamespace(token.to_owned()))
    }

    fn as_str(self) -> &'static str {
        match self {
            TemplateNamespace::Subject => SUBJECT_NAMESPACE,
            TemplateNamespace::Circumstance => CIRCUMSTANCE_NAMESPACE,
        }
    }
}

/// One subject/circumstance value available to selector templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateBinding {
    /// Binding namespace.
    pub namespace: TemplateNamespace,
    /// Stable field path inside the namespace.
    pub path: String,
    /// Resolved string value.
    pub value: String,
}

/// Subject/circumstance values available when resolving selector templates.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateBindings {
    /// Ordered, typed bindings.
    pub entries: Vec<TemplateBinding>,
}

impl TemplateBindings {
    /// Build bindings from ordered key/value pairs.
    #[must_use]
    pub fn from_pairs(
        subject: impl IntoIterator<Item = (String, String)>,
        circumstance: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        let mut entries = Vec::new();
        entries.extend(subject.into_iter().map(|(path, value)| TemplateBinding {
            namespace: TemplateNamespace::Subject,
            path,
            value,
        }));
        entries.extend(
            circumstance
                .into_iter()
                .map(|(path, value)| TemplateBinding {
                    namespace: TemplateNamespace::Circumstance,
                    path,
                    value,
                }),
        );
        Self { entries }
    }
}

/// Errors raised by template validation/resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError {
    /// The expression contains `${` without a closing `}`.
    Unterminated,
    /// The variable token is empty or malformed.
    InvalidToken(String),
    /// The variable namespace is not supported.
    UnsupportedNamespace(String),
    /// The referenced path does not exist in the provided bindings.
    MissingPath {
        /// Missing namespace.
        namespace: TemplateNamespace,
        /// Missing field path.
        path: String,
    },
    /// The referenced path resolves to an empty value.
    EmptyValue {
        /// Empty-value namespace.
        namespace: TemplateNamespace,
        /// Empty field path.
        path: String,
    },
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::Unterminated => write!(f, "unterminated template expression"),
            TemplateError::InvalidToken(token) => write!(f, "invalid template token {token}"),
            TemplateError::UnsupportedNamespace(namespace) => {
                write!(f, "unsupported template namespace {namespace}")
            }
            TemplateError::MissingPath { namespace, path } => {
                write!(f, "missing template path {}.{path}", namespace.as_str())
            }
            TemplateError::EmptyValue { namespace, path } => {
                write!(f, "empty template value {}.{path}", namespace.as_str())
            }
        }
    }
}

impl std::error::Error for TemplateError {}

/// Validate template syntax without resolving paths.
///
/// # Errors
/// Returns [`TemplateError`] when a variable is malformed or uses an unsupported
/// namespace.
#[must_use = "template validation errors must be handled"]
pub fn validate_template_expression(expression: &str) -> Result<(), TemplateError> {
    let mut tail = expression;
    loop {
        let Some(start) = tail.find("${") else {
            return Ok(());
        };
        let Some(variable) = tail.get(start + 2..) else {
            return Err(TemplateError::InvalidToken(String::new()));
        };
        let Some(end) = variable.find('}') else {
            return Err(TemplateError::Unterminated);
        };
        let Some(token) = variable.get(..end) else {
            return Err(TemplateError::InvalidToken(String::new()));
        };
        validate_token(token)?;
        let Some(next_tail) = variable.get(end + 1..) else {
            return Err(TemplateError::InvalidToken(token.to_owned()));
        };
        tail = next_tail;
    }
}

/// Resolve `${subject.*}` and `${circumstance.*}` placeholders exactly.
///
/// # Errors
/// Returns [`TemplateError`] for malformed expressions, unresolved variables or
/// empty resolved values.
#[must_use = "template resolution errors must be handled"]
pub fn resolve_template(
    expression: &str,
    bindings: &TemplateBindings,
) -> Result<String, TemplateError> {
    let mut tail = expression;
    let mut output = String::new();
    loop {
        let Some(start) = tail.find("${") else {
            output.push_str(tail);
            return Ok(output);
        };
        let Some(prefix) = tail.get(..start) else {
            return Err(TemplateError::InvalidToken(String::new()));
        };
        output.push_str(prefix);
        let Some(variable) = tail.get(start + 2..) else {
            return Err(TemplateError::InvalidToken(String::new()));
        };
        let Some(end) = variable.find('}') else {
            return Err(TemplateError::Unterminated);
        };
        let Some(token) = variable.get(..end) else {
            return Err(TemplateError::InvalidToken(String::new()));
        };
        let (namespace, path) = validate_token(token)?;
        let value = lookup_value(namespace, path, bindings)?;
        output.push_str(value);
        let Some(next_tail) = variable.get(end + 1..) else {
            return Err(TemplateError::InvalidToken(token.to_owned()));
        };
        tail = next_tail;
    }
}

fn validate_token(token: &str) -> Result<(TemplateNamespace, &str), TemplateError> {
    let Some((namespace_token, path)) = token.split_once('.') else {
        return Err(TemplateError::InvalidToken(token.to_owned()));
    };
    if namespace_token.is_empty() || path.is_empty() {
        return Err(TemplateError::InvalidToken(token.to_owned()));
    }
    let namespace = TemplateNamespace::from_token(namespace_token)?;
    Ok((namespace, path))
}

fn lookup_value<'a>(
    namespace: TemplateNamespace,
    path: &str,
    bindings: &'a TemplateBindings,
) -> Result<&'a str, TemplateError> {
    let value = bindings
        .entries
        .iter()
        .find(|entry| entry.namespace == namespace && entry.path == path)
        .map(|entry| entry.value.as_str());
    let Some(value) = value else {
        return Err(TemplateError::MissingPath {
            namespace,
            path: path.to_owned(),
        });
    };
    if value.is_empty() {
        return Err(TemplateError::EmptyValue {
            namespace,
            path: path.to_owned(),
        });
    }
    Ok(value)
}
