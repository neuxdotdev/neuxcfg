use crate::NeuxcfgError;
use std::fs;
use std::path::{Path, PathBuf};

/// Creates a backup of the given configuration file.
///
/// Copies `config_path` to `config_path` with the extension `.cfg.bak`.
/// On Unix, the backup file receives permissions `0600`.
///
/// # Errors
///
/// Returns [`NeuxcfgError::ProjectNotFound`] if `config_path` does not exist.
///
/// # Examples
///
/// ```no_run
/// use neuxcfg::backup::backup_project_config;
/// use std::path::Path;
///
/// let original = Path::new("/tmp/project.cfg");
/// std::fs::write(original, "key = \"val\"")?;
/// let backup = backup_project_config(original)?;
/// assert_eq!(backup.extension().unwrap(), "bak");
/// assert!(backup.exists());
/// # Ok::<(), neuxcfg::NeuxcfgError>(())
/// ```
pub fn backup_project_config(config_path: &Path) -> Result<PathBuf, NeuxcfgError> {
    if !config_path.exists() {
        return Err(NeuxcfgError::ProjectNotFound(
            config_path.to_string_lossy().into(),
        ));
    }
    let backup_path = config_path.with_extension("cfg.bak");
    fs::copy(config_path, &backup_path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&backup_path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(backup_path)
}

/// Restores a configuration file from its backup.
///
/// Copies `<config_path>.cfg.bak` back to `config_path`. On Unix, the
/// restored file receives permissions `0600`.
///
/// # Errors
///
/// Returns [`NeuxcfgError::ProjectNotFound`] if the backup file does not exist.
///
/// # Examples
///
/// ```no_run
/// use neuxcfg::backup::{backup_project_config, restore_project_config};
/// use std::path::Path;
///
/// let original = Path::new("/tmp/settings.cfg");
/// std::fs::write(original, "initial")?;
/// let _backup = backup_project_config(original)?;
/// std::fs::write(original, "modified")?;
/// restore_project_config(original)?;
/// assert_eq!(std::fs::read_to_string(original)?, "initial");
/// # Ok::<(), neuxcfg::NeuxcfgError>(())
/// ```
pub fn restore_project_config(config_path: &Path) -> Result<(), NeuxcfgError> {
    let backup_path = config_path.with_extension("cfg.bak");
    if !backup_path.exists() {
        return Err(NeuxcfgError::ProjectNotFound(
            "backup file not found".into(),
        ));
    }
    fs::copy(&backup_path, config_path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(config_path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}
