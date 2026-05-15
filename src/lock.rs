use crate::NeuxcfgError;
use fs2::FileExt;
use std::fs::File;
use std::path::Path;

/// An advisory file lock that releases automatically on drop.
///
/// Wraps a [`std::fs::File`] and acquires either an exclusive or shared lock
/// using the [`fs2`] crate. The lock is released when the `FileLock` is dropped.
///
/// # Thread Safety
///
/// `FileLock` is not `Send` or `Sync` because it relies on process‑level file
/// locking. It is safe to use in a single‑threaded context or behind proper
/// synchronisation primitives.
///
/// # Examples
///
/// ```no_run
/// use neuxcfg::lock::FileLock;
/// use std::path::Path;
///
/// let path = Path::new("/tmp/example.cfg");
/// std::fs::write(path, "data")?;
/// {
///     let _lock = FileLock::lock_exclusive(path)?;
///     // Safe to modify the file here.
/// }
/// // Lock released.
/// # Ok::<(), neuxcfg::NeuxcfgError>(())
/// ```
pub struct FileLock {
    file: File,
}

impl FileLock {
    /// Acquires an exclusive (write) lock on the file at `path`.
    ///
    /// The file must already exist. Blocks until the lock is available.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::LockError`] if the lock cannot be acquired,
    /// or [`NeuxcfgError::Io`] if the file cannot be opened.
    pub fn lock_exclusive(path: &Path) -> Result<Self, NeuxcfgError> {
        let file = File::open(path)?;
        file.lock_exclusive()?;
        Ok(Self { file })
    }

    /// Acquires a shared (read) lock on the file at `path`.
    ///
    /// The file must already exist. Blocks until the lock is available.
    ///
    /// # Errors
    ///
    /// Returns [`NeuxcfgError::LockError`] if the lock cannot be acquired,
    /// or [`NeuxcfgError::Io`] if the file cannot be opened.
    pub fn lock_shared(path: &Path) -> Result<Self, NeuxcfgError> {
        let file = File::open(path)?;
        file.lock_shared()?;
        Ok(Self { file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
