use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use fs2::FileExt;

pub struct Mutex(fs::File);

impl Drop for Mutex {
    fn drop(&mut self) {
        self.0.unlock().ok();
    }
}

/// Create a new mutex at the given path and attempt to acquire it.
pub fn acquire(path: impl AsRef<Path>) -> Result<Mutex> {
    let path = path.as_ref();
    let file = fs::OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("failed to open `{}`", path.display()))?;
    file.try_lock_exclusive()
        .with_context(|| format!("failed to acquire file lock `{}`", path.display()))?;
    Ok(Mutex(file))
}
