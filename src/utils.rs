use std::{fs, path::PathBuf};

/// A guard that automatically removes a lock file when dropped
pub struct LockGuard {
    lock_file: PathBuf,
}

impl LockGuard {
    pub fn new(lock_file: PathBuf) -> Self {
        Self { lock_file }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_file);
    }
}
