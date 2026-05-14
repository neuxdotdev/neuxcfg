use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum NeuxcfgError {
    #[error("cannot determine config directory")]
    ConfigDirNotFound,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("invalid project name: {0}")]
    InvalidProjectName(String),
    #[error("project '{0}' already exists")]
    ProjectAlreadyExists(String),
    #[error("project '{0}' not found")]
    ProjectNotFound(String),
    #[error("TOML parse error: {0}")]
    TomlParse(String),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(String),
    #[error("path traversal detected for project '{0}'")]
    PathTraversal(String),
    #[error("validation error: {0}")]
    ValidationError(String),
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
