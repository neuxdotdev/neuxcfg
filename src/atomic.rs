use crate::NeuxcfgError;
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

/// Writes `content` to `target` atomically.
///
/// A temporary file is created in the same directory as `target`, the content
/// is written and flushed, and then the file is renamed over `target`. This
/// guarantees that readers never see a partially written file.
///
/// The temporary file name incorporates a random UUID to avoid collisions.
/// On Unix, the temporary file is given permissions `0600`.
///
/// # Errors
///
/// Returns [`NeuxcfgError::Io`] if:
/// - `target` has no parent directory.
/// - The temporary file cannot be created, written, flushed, or renamed.
///
/// # Platform Support
///
/// On Unix, file permissions are explicitly set to `0600` for the temporary
/// file. On other platforms, default permissions apply.
///
/// # Examples
///
/// ```no_run
/// use neuxcfg::atomic::atomic_write;
/// use std::path::Path;
///
/// let path = Path::new("/tmp/config.cfg");
/// atomic_write(path, "key = \"value\"")?;
/// assert_eq!(std::fs::read_to_string(path)?, "key = \"value\"");
/// # Ok::<(), neuxcfg::NeuxcfgError>(())
/// ```
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
