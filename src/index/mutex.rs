use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use fs2::{lock_contended_error, FileExt};

use crate::index::FILES;

struct Mutex(fs::File);

impl Drop for Mutex {
    fn drop(&mut self) {
        self.0.unlock().ok();
    }
}

/// Create a new mutex at the given path and attempt to acquire it.
fn acquire(path: &Path) -> Result<Option<Mutex>> {
    let file = fs::OpenOptions::new()
        .read(true)
        .open(path)
        .with_context(|| format!("failed to open `{}`", path.display()))?;
    match file.try_lock_exclusive() {
        Ok(_) => Ok(Some(Mutex(file))),
        Err(err) if err.kind() == lock_contended_error().kind() => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("failed to acquire file lock `{}`", path.display()))
        }
    }
}

/// Execute the given function if we are able to acquire the mutex, else exit
/// immediately.
pub fn or_ignore<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if let Some(_mutex) = acquire(FILES.cache_dir())? {
        f()?;
    }
    Ok(())
}
