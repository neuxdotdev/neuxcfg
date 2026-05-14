use thiserror::Error;
#[derive(Error, Debug, PartialEq)]
pub enum NeuxcfgError {
    #[error("cannot determine config directory")]
    ConfigDirNotFound,
    #[error("I/O error: {0}")]
    Io(String),
}
impl From<std::io::Error> for NeuxcfgError {
    fn from(err: std::io::Error) -> Self {
        NeuxcfgError::Io(err.to_string())
    }
}