use crate::NeuxcfgError;
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;
pub fn atomic_write(target: &Path, content: &str) -> Result<(), NeuxcfgError> {
    let parent = target
        .parent()
        .ok_or_else(|| NeuxcfgError::Io("target has no parent directory".into()))?;
    let tmp_name = format!(
        ".{}.tmp.{}",
        target.file_name().unwrap().to_string_lossy(),
        Uuid::new_v4()
    );
    let tmp_path = parent.join(tmp_name);
    let mut tmp_file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)?;
    tmp_file.write_all(content.as_bytes())?;
    tmp_file.flush()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(&tmp_path, target)?;
    Ok(())
}
