mod error;
pub use error::NeuxcfgError;
use std::path::{Path, PathBuf};
pub struct Neuxcfg {
    root: PathBuf,
}
impl Neuxcfg {
    pub fn new() -> Result<Self, NeuxcfgError> {
        let config_dir = dirs::config_dir().ok_or(NeuxcfgError::ConfigDirNotFound)?;
        Ok(Self {
            root: config_dir.join("neuxcfg"),
        })
    }
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn init(&self) -> Result<(), NeuxcfgError> {
        if !self.root.exists() {
            std::fs::create_dir_all(&self.root)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&self.root, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        let config_file = self.root.join("config.cfg");
        if !config_file.exists() {
            std::fs::write(&config_file, "")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&config_file, std::fs::Permissions::from_mode(0o600))?;
            }
        }
        Ok(())
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
}
