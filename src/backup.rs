use crate::NeuxcfgError;
use std::fs;
use std::path::{Path, PathBuf};
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
