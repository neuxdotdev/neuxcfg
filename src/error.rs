use thiserror::Error;

/// Represents all possible errors that can occur in the neuxcfg library.
///
/// The error type is designed to give clear, actionable messages and supports
/// conversion from standard I/O errors and TOML parse/serialization errors.
#[derive(Error, Debug, PartialEq)]
pub enum NeuxcfgError {
    /// The platform's configuration directory could not be determined.
    ///
    /// This may happen on systems where `dirs::config_dir()` returns `None`,
    /// e.g., some embedded or restricted environments.
    #[error("cannot determine config directory")]
    ConfigDirNotFound,

    /// An I/O operation failed.
    ///
    /// The string contains the operating system's error message.
    #[error("I/O error: {0}")]
    Io(String),

    /// The provided project name is invalid.
    ///
    /// Valid names must be non‑empty and must not contain `/`, `\`, `..`, or null bytes.
    #[error("invalid project name: {0}")]
    InvalidProjectName(String),

    /// A project with the given name already exists.
    #[error("project '{0}' already exists")]
    ProjectAlreadyExists(String),

    /// The requested project does not exist.
    #[error("project '{0}' not found")]
    ProjectNotFound(String),

    /// Failed to parse TOML content.
    ///
    /// The string contains the detailed parse error message.
    #[error("TOML parse error: {0}")]
    TomlParse(String),

    /// Failed to serialize a value to TOML.
    #[error("TOML serialize error: {0}")]
    TomlSerialize(String),

    /// A path traversal attempt was detected.
    ///
    /// This occurs when a resolved project path escapes the neuxcfg root directory.
    #[error("path traversal detected for project '{0}'")]
    PathTraversal(String),
}

/// Converts a standard I/O error into a `NeuxcfgError::Io` variant,
/// preserving the error message.
impl From<std::io::Error> for NeuxcfgError {
    fn from(err: std::io::Error) -> Self {
        NeuxcfgError::Io(err.to_string())
    }
}

/// Converts a TOML deserialisation error into a `NeuxcfgError::TomlParse` variant.
impl From<toml::de::Error> for NeuxcfgError {
    fn from(err: toml::de::Error) -> Self {
        NeuxcfgError::TomlParse(err.to_string())
    }
}

/// Converts a TOML serialisation error into a `NeuxcfgError::TomlSerialize` variant.
impl From<toml::ser::Error> for NeuxcfgError {
    fn from(err: toml::ser::Error) -> Self {
        NeuxcfgError::TomlSerialize(err.to_string())
    }
}
