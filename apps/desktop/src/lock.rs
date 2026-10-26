use std::fs::{File, TryLockError};
use std::path::Path;

use anyhow::{Context, Result};

use crate::app;

pub fn try_lock_screen() -> Result<Option<File>> {
    try_lock(&app::runtime_dir()?.join("screen.lock"))
}

fn try_lock(path: &Path) -> Result<Option<File>> {
    let file = File::create(path).with_context(|| format!("open {}", path.display()))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(e)) => Err(e).with_context(|| format!("lock {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_holder_at_a_time() {
        let path = std::env::temp_dir().join(format!("clip-lock-test-{}", std::process::id()));
        let first = try_lock(&path).unwrap();
        assert!(first.is_some());
        assert!(try_lock(&path).unwrap().is_none());
        drop(first);
        assert!(try_lock(&path).unwrap().is_some());
        std::fs::remove_file(path).ok();
    }
}
