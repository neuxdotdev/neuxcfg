mod error;
pub mod types;
pub use error::NeuxcfgError;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
pub use types::{GlobalConfig, ProjectConfig};
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
            Ok(mut file) => {
                let global_config = GlobalConfig::from_cargo();
                let toml_str = toml::to_string_pretty(&global_config)?;
                file.write_all(toml_str.as_bytes())?;
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
    fn validate_project_name(name: &str) -> Result<(), NeuxcfgError> {
        if name.is_empty() {
            return Err(NeuxcfgError::InvalidProjectName(
                "name cannot be empty".into(),
            ));
        }
        if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('\0') {
            return Err(NeuxcfgError::InvalidProjectName(format!(
                "invalid characters in '{}'",
                name
            )));
        }
        Ok(())
    }
    fn secure_project_dir(&self, name: &str) -> Result<PathBuf, NeuxcfgError> {
        Self::validate_project_name(name)?;
        let proj_dir = self.root.join(name);
        if proj_dir.exists() {
            let canonical_proj = proj_dir.canonicalize()?;
            let canonical_root = self.root.canonicalize()?;
            if !canonical_proj.starts_with(&canonical_root) {
                return Err(NeuxcfgError::PathTraversal(name.to_string()));
            }
            Ok(canonical_proj)
        } else {
            Ok(proj_dir)
        }
    }
    fn project_config_path(&self, name: &str) -> Result<PathBuf, NeuxcfgError> {
        let dir = self.secure_project_dir(name)?;
        Ok(dir.join(format!("{}.config.cfg", name)))
    }
    pub fn add_project(&self, name: &str) -> Result<(), NeuxcfgError> {
        Self::validate_project_name(name)?;
        let proj_dir = self.root.join(name);
        if proj_dir.exists() {
            return Err(NeuxcfgError::ProjectAlreadyExists(name.to_string()));
        }
        std::fs::create_dir(&proj_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&proj_dir, std::fs::Permissions::from_mode(0o700))?;
        }
        let default_config =
            ProjectConfig::new(name.to_string(), proj_dir.to_string_lossy().into());
        let config_path = proj_dir.join(format!("{}.config.cfg", name));
        let toml_str = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, toml_str)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
    pub fn remove_project(&self, name: &str) -> Result<(), NeuxcfgError> {
        let proj_dir = self.secure_project_dir(name)?;
        if !proj_dir.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        std::fs::remove_dir_all(&proj_dir)?;
        Ok(())
    }
    pub fn list_projects(&self) -> Result<Vec<String>, NeuxcfgError> {
        let mut projects = Vec::new();
        if !self.root.exists() {
            return Ok(projects);
        }
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                if name == "." || name == ".." {
                    continue;
                }
                if Self::validate_project_name(name).is_ok() {
                    projects.push(name.to_string());
                }
            }
        }
        Ok(projects)
    }
    pub fn get_project_config(&self, name: &str) -> Result<ProjectConfig, NeuxcfgError> {
        let config_path = self.project_config_path(name)?;
        if !config_path.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }
    pub fn set_project_config(
        &self,
        name: &str,
        config: &ProjectConfig,
    ) -> Result<(), NeuxcfgError> {
        let config_path = self.project_config_path(name)?;
        if !config_path.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        let toml_str = toml::to_string_pretty(config)?;
        std::fs::write(&config_path, toml_str)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
    pub fn project_exists(&self, name: &str) -> Result<bool, NeuxcfgError> {
        Self::validate_project_name(name)?;
        Ok(self.root.join(name).is_dir())
    }
}
