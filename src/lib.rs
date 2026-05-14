mod error;
pub use error::NeuxcfgError;
use std::io::ErrorKind;
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
        std::fs::create_dir_all(&self.root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.root, std::fs::Permissions::from_mode(0o700))?;
        }
        let config_path = self.root.join("config.cfg");
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)  
            .open(&config_path)
        {
            Ok(_file) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
                }
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
}