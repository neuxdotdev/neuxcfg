use thiserror::Error;

/// Errors that can occur in the `neuxcfg` library.
///
/// Each variant represents a distinct failure mode, often carrying contextual
/// information as a `String`. Conversions from common error types are provided
/// via [`From`] implementations.
///
/// # Examples
///
/// ```rust
/// use neuxcfg::NeuxcfgError;
///
/// let err = NeuxcfgError::InvalidProjectName("bad/name".into());
/// assert_eq!(err.to_string(), "invalid project name: bad/name");
/// ```
#[derive(Error, Debug, PartialEq)]
pub enum NeuxcfgError {
    /// The system configuration directory could not be determined.
    #[error("cannot determine config directory")]
    ConfigDirNotFound,

    /// Wraps an I/O error with its message.
    #[error("I/O error: {0}")]
    Io(String),

    /// A project name contained illegal characters or was empty.
    #[error("invalid project name: {0}")]
    InvalidProjectName(String),

    /// Attempted to add a project that already exists.
    #[error("project '{0}' already exists")]
    ProjectAlreadyExists(String),

    /// The requested project does not exist.
    #[error("project '{0}' not found")]
    ProjectNotFound(String),

    /// Failed to parse a TOML configuration file.
    #[error("TOML parse error: {0}")]
    TomlParse(String),

    /// Failed to serialize a value to TOML.
    #[error("TOML serialize error: {0}")]
    TomlSerialize(String),

    /// A path traversal attack was detected.
    #[error("path traversal detected for project '{0}'")]
    PathTraversal(String),

    /// Custom validation failed (e.g., invalid extra field).
    #[error("validation error: {0}")]
    ValidationError(String),

    /// An advisory file lock could not be acquired.
    #[error("lock error: {0}")]
    LockError(String),
}

impl From<std::io::Error> for NeuxcfgError {
    fn from(err: std::io::Error) -> Self {
        NeuxcfgError::Io(err.to_string())
    }
}

impl From<toml::de::Error> for NeuxcfgError {
    fn from(err: toml::de::Error) -> Self {
        NeuxcfgError::TomlParse(err.to_string())
    }
}

impl From<toml::ser::Error> for NeuxcfgError {
    fn from(err: toml::ser::Error) -> Self {
        NeuxcfgError::TomlSerialize(err.to_string())
    }
}

impl From<uuid::Error> for NeuxcfgError {
    fn from(err: uuid::Error) -> Self {
        NeuxcfgError::Io(format!("uuid generation failed: {}", err))
    }
}
