//! # neuxcfg
//!
//! A production-ready library for secure project configuration management.
//!
//! `neuxcfg` provides a robust foundation for storing, retrieving, and modifying
//! per-project settings. It ensures data integrity through atomic writes, file
//! locking, automatic backups, and strict validation. Path traversal attacks are
//! prevented by canonicalization checks.
//!
//! ## Quick Start
//!
//! ```no_run
//! use neuxcfg::Neuxcfg;
//!
//! let cfg = Neuxcfg::new()?;
//! cfg.init()?;
//! cfg.add_project("my_app")?;
//! let config = cfg.get_project_config("my_app")?;
//! println!("Project name: {}", config.project.name);
//! # Ok::<(), neuxcfg::NeuxcfgError>(())
//! ```
//!
//! ## Features
//!
//! - **Atomic configuration writes** – prevents corruption via UUID-named temp files.
//! - **File locking** – shared/exclusive locks via `fs2` for safe concurrent access.
//! - **Backup and restore** – automatic `.bak` creation before modifications.
//! - **Path security** – directory traversal protection via `canonicalize`.
//! - **Validation engine** – enforces allowed extra field types and key formats.
//! - **Deep merge** – update configurations partially without full replacement.
//! - **Unix permission hardening** – directories `0700`, files `0600` on Unix.

pub mod atomic;
pub mod backup;
pub mod error;
pub mod lock;
pub mod merge;
pub mod types;
pub mod validate;

use atomic::atomic_write;
use backup::backup_project_config;
pub use error::NeuxcfgError;
use lock::FileLock;
use merge::deep_merge;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
pub use types::{GlobalConfig, ProjectConfig};
use validate::validate_extra;

/// Central configuration manager for `neuxcfg`.
///
/// `Neuxcfg` stores all project configurations under a single root directory.
/// The default root is `<system_config_dir>/neuxcfg` (e.g., `~/.config/neuxcfg`
/// on Linux). Use [`with_root`](Self::with_root) for custom locations.
///
/// # Thread Safety
///
/// `Neuxcfg` is neither `Send` nor `Sync` because it does not implement
/// automatic synchronization. However, all file operations are protected by
/// advisory file locks when reading/writing project configurations.
///
/// # Examples
///
/// ```no_run
/// use neuxcfg::Neuxcfg;
///
/// // Use system config directory.
/// let cfg = Neuxcfg::new()?;
/// cfg.init()?;
/// cfg.add_project("web_server")?;
/// # Ok::<(), neuxcfg::NeuxcfgError>(())
/// ```
///
/// With a custom root:
///
/// ```no_run
/// use neuxcfg::Neuxcfg;
/// use std::path::PathBuf;
///
/// let cfg = Neuxcfg::with_root(PathBuf::from("/tmp/test_config"));
/// cfg.init()?;
/// # Ok::<(), neuxcfg::NeuxcfgError>(())
/// ```
#[derive(Debug)]
pub struct Neuxcfg {
    root: PathBuf,
}

impl Neuxcfg {
    /// Creates a new `Neuxcfg` instance using the system configuration directory.
    ///
    /// The root is set to `<config_dir>/neuxcfg`, where `<config_dir>` is obtained
    /// from the [`dirs`](https://crates.io/crates/dirs) crate.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ConfigDirNotFound`] if the system configuration
    /// directory cannot be determined (rare on supported platforms).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn new() -> Result<Self, NeuxcfgError> {
        let config_dir = dirs::config_dir().ok_or(NeuxcfgError::ConfigDirNotFound)?;
        Ok(Self {
            root: config_dir.join("neuxcfg"),
        })
    }

    /// Creates a new `Neuxcfg` instance with an explicit root path.
    ///
    /// This constructor does **not** create the directory; call [`init`](Self::init)
    /// afterwards to initialise the root.
    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    /// Initialises the root directory and writes the default global configuration.
    ///
    /// If the root directory does not exist, it is created with permissions `0700`
    /// on Unix. A `config.cfg` file is written only if it does not already exist.
    /// On Unix, this file receives permissions `0600`.
    ///
    /// The global configuration is populated from Cargo environment variables
    /// (e.g., `CARGO_PKG_NAME`, `CARGO_PKG_VERSION`). If those variables are
    /// unavailable, sensible defaults are used.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::Io`] if directory creation or file writing fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
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

    /// Returns the root directory path.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::with_root("/tmp/cfg".into());
    /// assert_eq!(cfg.root(), std::path::Path::new("/tmp/cfg"));
    /// ```
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Validates that a project name does not contain path separators or `..`.
    ///
    /// This is used internally before any file‑system operation on a project
    /// directory.
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

    /// Returns the canonical path of a project directory, preventing path traversal.
    ///
    /// If the directory already exists, it is canonicalised and checked to be
    /// within the root. Otherwise, the (non‑existing) path is returned directly.
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

    /// Builds the path to the configuration file of a project.
    ///
    /// The file is named `<project_name>.config.cfg` inside the project directory.
    fn project_config_path(&self, name: &str) -> Result<PathBuf, NeuxcfgError> {
        let dir = self.secure_project_dir(name)?;
        Ok(dir.join(format!("{}.config.cfg", name)))
    }

    /// Adds a new project with a default configuration.
    ///
    /// Creates the project directory (Unix permissions `0700`), validates the
    /// default extra fields, and writes the initial `ProjectConfig` atomically.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectAlreadyExists`] if the project directory
    /// already exists. Returns [`NeuxcfgError::InvalidProjectName`] if the name
    /// is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("my_project")?;
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
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
        validate_extra(&default_config.project.extra)?;
        let config_path = proj_dir.join(format!("{}.config.cfg", name));
        let toml_str = toml::to_string_pretty(&default_config)?;
        atomic_write(&config_path, &toml_str)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    /// Removes a project and all its configuration files permanently.
    ///
    /// Deletes the entire project directory tree.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the project does not exist.
    /// Returns [`NeuxcfgError::PathTraversal`] if the resolved path escapes the root.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("temp")?;
    /// cfg.remove_project("temp")?;
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn remove_project(&self, name: &str) -> Result<(), NeuxcfgError> {
        let proj_dir = self.secure_project_dir(name)?;
        if !proj_dir.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        std::fs::remove_dir_all(&proj_dir)?;
        Ok(())
    }

    /// Lists all valid project names stored under the root.
    ///
    /// Only directories whose names pass [`validate_project_name`](Self::validate_project_name)
    /// are included.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::Io`] if the root directory cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("alpha")?;
    /// let projects = cfg.list_projects()?;
    /// assert!(projects.contains(&"alpha".to_string()));
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
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

    /// Checks whether a project directory exists.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::InvalidProjectName`] if the name is invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// assert!(!cfg.project_exists("nonexistent")?);
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn project_exists(&self, name: &str) -> Result<bool, NeuxcfgError> {
        Self::validate_project_name(name)?;
        Ok(self.root.join(name).is_dir())
    }

    /// Reads the configuration of a project.
    ///
    /// A shared file lock is acquired during the read to prevent concurrent
    /// modifications.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the project or its config
    /// file does not exist. Returns [`NeuxcfgError::TomlParse`] if the file
    /// content is not valid TOML.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("demo")?;
    /// let config = cfg.get_project_config("demo")?;
    /// println!("Project path: {}", config.project.path);
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn get_project_config(&self, name: &str) -> Result<ProjectConfig, NeuxcfgError> {
        let config_path = self.project_config_path(name)?;
        if !config_path.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        let _lock = FileLock::lock_shared(&config_path)
            .map_err(|e| NeuxcfgError::LockError(e.to_string()))?;
        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Overwrites the entire project configuration.
    ///
    /// Before writing, a backup of the current configuration is created
    /// (see [`backup_project`](Self::backup_project)). The write is atomic and
    /// protected by an exclusive file lock.
    ///
    /// The `extra` fields of the provided configuration are validated before
    /// writing.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the project does not exist.
    /// Returns [`NeuxcfgError::ValidationError`] if `extra` fields are invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::{Neuxcfg, ProjectConfig};
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("app")?;
    ///
    /// let new_config = ProjectConfig::new("app".into(), "/opt/app".into());
    /// cfg.set_project_config("app", &new_config)?;
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn set_project_config(
        &self,
        name: &str,
        config: &ProjectConfig,
    ) -> Result<(), NeuxcfgError> {
        validate_extra(&config.project.extra)?;
        let config_path = self.project_config_path(name)?;
        if !config_path.exists() {
            return Err(NeuxcfgError::ProjectNotFound(name.to_string()));
        }
        backup_project_config(&config_path)?;
        let _lock = FileLock::lock_exclusive(&config_path)
            .map_err(|e| NeuxcfgError::LockError(e.to_string()))?;
        let toml_str = toml::to_string_pretty(config)?;
        atomic_write(&config_path, &toml_str)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    /// Merges a TOML delta into an existing project configuration.
    ///
    /// The current configuration is read, deserialised to a [`toml::Value`],
    /// deeply merged with the provided `delta`, and then written back atomically.
    /// This allows partial updates without constructing a full `ProjectConfig`.
    ///
    /// # Errors
    ///
    /// Can return any error from [`get_project_config`](Self::get_project_config)
    /// or [`set_project_config`](Self::set_project_config), including validation errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    /// use toml::toml;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("server")?;
    ///
    /// let delta = toml! {
    ///     [project]
    ///     debug = true
    ///     limits = { max_connections = 100 }
    /// };
    /// cfg.update_project_config("server", toml::Value::Table(delta))?;
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn update_project_config(
        &self,
        name: &str,
        delta: toml::Value,
    ) -> Result<(), NeuxcfgError> {
        let config = self.get_project_config(name)?;
        let config_str = toml::to_string(&config)?;
        let mut config_value: toml::Value = toml::from_str(&config_str)?;
        deep_merge(&mut config_value, &delta);
        let merged_str = toml::to_string(&config_value)?;
        let merged_config: ProjectConfig = toml::from_str(&merged_str)?;
        self.set_project_config(name, &merged_config)
    }

    /// Creates a backup of the project configuration file.
    ///
    /// The backup file has the extension `.cfg.bak` and is placed in the same
    /// directory.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the config file does not exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("important")?;
    /// let backup = cfg.backup_project("important")?;
    /// assert!(backup.extension().unwrap() == "bak");
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn backup_project(&self, name: &str) -> Result<PathBuf, NeuxcfgError> {
        let config_path = self.project_config_path(name)?;
        backup_project_config(&config_path)
    }

    /// Restores a project configuration from its backup file.
    ///
    /// The backup file must exist (`<config>.cfg.bak`). The current configuration
    /// is overwritten.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::ProjectNotFound`] if the backup file does not exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use neuxcfg::Neuxcfg;
    ///
    /// let cfg = Neuxcfg::new()?;
    /// cfg.init()?;
    /// cfg.add_project("data")?;
    /// cfg.backup_project("data")?;
    /// // ... modifications ...
    /// cfg.restore_project("data")?;
    /// # Ok::<(), neuxcfg::NeuxcfgError>(())
    /// ```
    pub fn restore_project(&self, name: &str) -> Result<(), NeuxcfgError> {
        let config_path = self.project_config_path(name)?;
        backup::restore_project_config(&config_path)
    }
}
