use thiserror::Error;
#[derive(Error, Debug)]
pub enum NeuxcfgError {
    #[error("cannot determine config directory")]
    ConfigDirNotFound,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}