use crate::NeuxcfgError;
use fs2::FileExt;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
pub struct FileLock {
    file: File,
}
impl FileLock {
    pub fn lock_exclusive(path: &Path) -> Result<Self, NeuxcfgError> {
        let file = File::open(path)?;
        file.lock_exclusive()?;
        Ok(Self { file })
    }
    pub fn lock_shared(path: &Path) -> Result<Self, NeuxcfgError> {
        let file = File::open(path)?;
        file.lock_shared()?;
        Ok(Self { file })
    }
    pub fn lock_exclusive_timeout(_path: &Path, _timeout: Duration) -> Result<Self, NeuxcfgError> {
        unimplemented!("timeout-based locking is not yet available")
    }
}
impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
