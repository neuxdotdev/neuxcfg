//! # neuxcfg
//!
//! A library for managing application configuration directories and per-project settings.
//!
//! The crate creates a root configuration directory (platform-appropriate, e.g., `~/.config/neuxcfg`
//! on Linux) and provides an API to initialize a global config file and manage
//! project-specific configuration files in TOML format. It focuses on security:
//! strict Unix permissions (0700 for directories, 0600 for files), path traversal
//! prevention, and validation of project names.
//!
//! ## Quick start
//!
//! ```rust
//! use neuxcfg::Neuxcfg;
//!
//! let cfg = Neuxcfg::new()?;
//! cfg.init()?;
//! cfg.add_project("my_app")?;
//! let config = cfg.get_project_config("my_app")?;
//! println!("{:?}", config);
//! # Ok::<(), neuxcfg::NeuxcfgError>(())
//! ```

mod error;
pub mod types;
pub use error::NeuxcfgError;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
pub use types::{GlobalConfig, ProjectConfig};

/// The central manager for neuxcfg.
///
/// Holds the root path where all configuration data is stored. The root defaults
/// to `$CONFIG_DIR/neuxcfg` (platform specific) but can be overridden for testing
/// via [`with_root`](Neuxcfg::with_root).
pub struct Neuxcfg {
    root: PathBuf,
}

impl Neuxcfg {
    /// Creates a new instance using the default configuration directory.
    ///
    /// The directory is obtained from `dirs::config_dir()`, which maps to:
    /// - Linux: `$XDG_CONFIG_HOME` or `~/.config`
    /// - macOS: `~/Library/Application Support`
    /// - Windows: `C:\Users\<user>\AppData\Roaming`
    /// - Android: `$HOME/.config` (if available)
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ConfigDirNotFound`] if the platform's config directory
    /// cannot be determined.
    pub fn new() -> Result<Self, NeuxcfgError> {
        let config_dir = dirs::config_dir().ok_or(NeuxcfgError::ConfigDirNotFound)?;
        Ok(Self {
            root: config_dir.join("neuxcfg"),
        })
    }

    /// Creates an instance with an arbitrary root path.
    ///
    /// Useful for testing or when you want to store configuration in a
    /// non‑standard location. The path is used exactly as given; no `neuxcfg`
    /// subdirectory is appended.
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Initialises the configuration root.
    ///
    /// Creates the root directory (with `create_dir_all`, so parent directories are
    /// created if missing) and the global `config.cfg` file if they do not yet exist.
    /// On Unix platforms, directory permissions are set to `0o700` and the global
    /// config file to `0o600` for security.
    ///
    /// The global config file is initialised with metadata extracted from the
    /// current crate’s environment variables (via [`GlobalConfig::from_cargo`]).
    /// If the file already exists, its content is left untouched; only permissions
    /// are re‑applied on Unix.
    ///
    /// This method is idempotent: calling it multiple times does not overwrite
    /// an existing global config file.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if directory creation or file operations fail.
    /// Also returns a TOML serialization error if the default global config
    /// cannot be serialized (unlikely because the struct derives `Serialize`).
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

    /// Returns a reference to the root path.
    ///
    /// This is the directory that contains the global `config.cfg` and all project
    /// subdirectories.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Validates a project name.
    ///
    /// A valid project name must not be empty and must not contain any of the
    /// following characters: `/`, `\`, `..`, or the null byte `\0`.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::InvalidProjectName`] if the name is invalid.
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

    /// Returns a canonical, safe path to a project directory.
    ///
    /// The project name is validated first. If the directory exists, the path is
    /// canonicalised and checked to ensure it resides within the root directory
    /// (path traversal protection). If the directory does **not** exist, a
    /// non‑canonical (but still valid) path is returned so that it can be
    /// created later.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::InvalidProjectName`] or
    /// [`NeuxcfgError::PathTraversal`] if validation fails.
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

    /// Returns the path to a project’s configuration file.
    ///
    /// The filename is `<project_name>.config.cfg` inside the project directory.
    /// This method internally calls [`secure_project_dir`](Neuxcfg::secure_project_dir)
    /// to ensure the path is safe.
    ///
    /// # Errors
    ///
    /// Same as `secure_project_dir`.
    fn project_config_path(&self, name: &str) -> Result<PathBuf, NeuxcfgError> {
        let dir = self.secure_project_dir(name)?;
        Ok(dir.join(format!("{}.config.cfg", name)))
    }

    /// Creates a new project configuration directory and file.
    ///
    /// The project name is validated. A new subdirectory is created inside the root
    /// with strict permissions (0700 on Unix). Inside that directory, a default
    /// `ProjectConfig` (containing the project name and path) is serialised to
    /// `<name>.config.cfg` with permissions 0600 on Unix.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::InvalidProjectName`] if the name is invalid,
    /// [`NeuxcfgError::ProjectAlreadyExists`] if the project directory already exists,
    /// or an I/O / serialisation error if the file cannot be written.
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

    /// Removes an entire project directory and its contents.
    ///
    /// The project must exist. Path traversal checks are performed.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the project does not exist,
    /// [`NeuxcfgError::InvalidProjectName`] or [`NeuxcfgError::PathTraversal`] if
    /// the name is invalid, or an I/O error if deletion fails.
    pub fn remove_project(&self, name: &str) -> Result<(), NeuxcfgError> {
        let proj_dir = self.secure_project_dir(name)?;
        if !proj_dir.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        std::fs::remove_dir_all(&proj_dir)?;
        Ok(())
    }

    /// Lists all currently existing project names.
    ///
    /// Only subdirectories of the root that pass validation are included.
    /// Entries named `.` or `..` are skipped.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if reading the root directory fails.
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

    /// Reads and deserialises the configuration of a project.
    ///
    /// The project’s `<name>.config.cfg` file is read and parsed as TOML into a
    /// [`ProjectConfig`] struct.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the project or its config file
    /// does not exist, a parse error if the TOML is malformed, or an I/O error.
    pub fn get_project_config(&self, name: &str) -> Result<ProjectConfig, NeuxcfgError> {
        let config_path = self.project_config_path(name)?;
        if !config_path.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Overwrites the configuration file of a project with the given `ProjectConfig`.
    ///
    /// The existing file is completely replaced. On Unix, permissions are reset to
    /// 0600 after writing.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the project does not exist,
    /// or an I/O / serialisation error.
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

    /// Checks whether a project directory exists.
    ///
    /// Returns `true` if the directory exists and is a directory. The name is
    /// validated first; if invalid, an error is returned.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::InvalidProjectName`] if the name is invalid.
    pub fn project_exists(&self, name: &str) -> Result<bool, NeuxcfgError> {
        Self::validate_project_name(name)?;
        Ok(self.root.join(name).is_dir())
    }
}
