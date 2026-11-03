use serde::Serialize;
use std::{collections::BTreeMap, fmt, io, path::PathBuf};

/// Stable diagnostic severity used by library and CLI consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// Identifies which contract produced a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticDomain {
    ApplicationValidation,
    CapabilityEvidence,
    FirmwareCompatibility,
    Repository,
    Serialization,
    Cli,
}

/// Structured, machine-readable diagnostic. `code` is a documented stable identifier.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub domain: DiagnosticDomain,
    pub message: String,
    pub profile_id: Option<String>,
    pub path: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub context: BTreeMap<String, serde_json::Value>,
    pub remediation: Option<String>,
}

impl Diagnostic {
    pub fn application(
        code: &'static str,
        profile_id: Option<&str>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: DiagnosticSeverity::Error,
            domain: DiagnosticDomain::ApplicationValidation,
            message: message.into(),
            profile_id: profile_id.map(str::to_owned),
            path: Some(path.into()),
            context: BTreeMap::new(),
            remediation: None,
        }
    }
}

/// Backwards-compatible name for Phase 10 validation consumers.
pub type ValidationIssue = Diagnostic;

/// Errors returned by profile serialization and repository operations.
#[derive(Debug)]
pub enum CoreError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidProfile(Vec<ValidationIssue>),
    AlreadyExists(PathBuf),
    NotFound(String),
    InvalidId(String),
    UnsupportedSchema(u32),
    /// Another process currently owns the profile mutation lock.
    Busy(String),
    /// The profile changed after it was loaded; refusing a stale update.
    Conflict(String),
}

impl CoreError {
    /// Stable machine-readable code for this error category.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Io(_) => "REPOSITORY_IO_ERROR",
            Self::Json(_) => "PROFILE_JSON_INVALID",
            Self::InvalidProfile(_) => "PROFILE_VALIDATION_FAILED",
            Self::AlreadyExists(_) => "PROFILE_ALREADY_EXISTS",
            Self::NotFound(_) => "PROFILE_NOT_FOUND",
            Self::InvalidId(_) => "PROFILE_ID_INVALID",
            Self::UnsupportedSchema(_) => "PROFILE_SCHEMA_UNSUPPORTED",
            Self::Busy(_) => "PROFILE_LOCK_BUSY",
            Self::Conflict(_) => "PROFILE_CONCURRENT_MODIFICATION",
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "profile I/O error: {e}"),
            Self::Json(e) => write!(f, "profile JSON error: {e}"),
            Self::InvalidProfile(items) => write!(
                f,
                "profile validation failed ({} issue(s)): {}",
                items.len(),
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            Self::AlreadyExists(p) => write!(f, "profile already exists: {}", p.display()),
            Self::NotFound(id) => write!(f, "profile not found: {id}"),
            Self::InvalidId(id) => write!(f, "invalid profile identifier: {id}"),
            Self::UnsupportedSchema(v) => write!(f, "unsupported profile schema version {v}"),
            Self::Busy(id) => write!(f, "profile is being modified by another process: {id}"),
            Self::Conflict(id) => write!(
                f,
                "profile changed since it was loaded; reload before saving: {id}"
            ),
        }
    }
}
impl std::error::Error for CoreError {}
impl From<io::Error> for CoreError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}: {}",
            self.code,
            self.path.as_deref().unwrap_or("<profile>"),
            self.message
        )
    }
}
