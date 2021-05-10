mod mutex;
mod nix;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Result};
use once_cell::sync::Lazy;

const UPDATE_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);
const CRATES_IO_INDEX: &str = "https://github.com/rust-lang/crates.io-index";
pub static FILES: Lazy<Files> = Lazy::new(Files::new);

pub struct Files {
    cache_dir: PathBuf,
    index_dir: PathBuf,
    update_file: PathBuf,
}

impl Files {
    fn new() -> Self {
        let cache_dir = env::var_os("alfred_workflow_cache")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let bundle_id = env::var("alfred_workflow_bundleid")
                    .map(Cow::from)
                    .unwrap_or_else(|_| Cow::from("io.macarthur.ross.crates"));
                home::home_dir()
                    .unwrap()
                    .join("Library/Caches/com.runningwithcrayons.Alfred/Workflow Data")
                    .join(&*bundle_id)
            });

        let index_dir = cache_dir.join("crates.io-index");
        let update_file = index_dir.join(".last-modified");

        Self {
            cache_dir,
            index_dir,
            update_file,
        }
    }

    fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn index_dir(&self) -> &Path {
        &self.index_dir
    }

    fn update_file(&self) -> &Path {
        &self.update_file
    }
}

fn git() -> process::Command {
    let mut cmd = process::Command::new("git");
    cmd.stdin(process::Stdio::null());
    cmd.stdout(process::Stdio::null());
    cmd.stderr(process::Stdio::null());
    cmd
}

fn git_clone(url: &str, path: impl AsRef<Path>) -> Result<()> {
    let output = git()
        .args(&["clone", "--depth", "1"])
        .arg(url)
        .arg(path.as_ref())
        .output()?;
    if !output.status.success() {
        bail!("failed to run `git clone` command");
    }
    Ok(())
}

fn git_fetch(path: impl AsRef<Path>) -> Result<()> {
    let output = git().arg("-C").arg(path.as_ref()).arg("fetch").output()?;
    if !output.status.success() {
        bail!("failed to run `git fetch` command");
    }
    Ok(())
}

fn git_reset(path: impl AsRef<Path>) -> Result<()> {
    let output = git()
        .arg("-C")
        .arg(path.as_ref())
        .args(&["reset", "--hard", "origin/HEAD"])
        .output()?;
    if !output.status.success() {
        bail!("failed to run `git reset` command");
    }
    Ok(())
}

fn download() -> Result<()> {
    fs::create_dir_all(FILES.cache_dir())?;
    let _mutex = mutex::acquire(FILES.cache_dir())?;
    git_clone(CRATES_IO_INDEX, FILES.index_dir())?;
    fs::File::create(FILES.update_file())?;
    Ok(())
}

fn update() -> Result<()> {
    let _mutex = mutex::acquire(FILES.cache_dir())?;
    git_fetch(FILES.index_dir())?;
    git_reset(FILES.index_dir())?;
    fs::File::create(FILES.update_file())?;
    Ok(())
}

/// Checks that the Crates.io index is okay and returns the path to it.
///
/// This function will spawn a subprocess to clone it if it missing or update it
/// if it is out of date.
pub fn check() -> Result<()> {
    if FILES.index_dir().exists() {
        let needs_update = match fs::metadata(FILES.update_file()) {
            Ok(metadata) => {
                let now = SystemTime::now();
                let then = metadata.modified()?;
                now.duration_since(then)? > UPDATE_INTERVAL
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => true,
            Err(err) => return Err(err.into()),
        };
        if needs_update {
            nix::exec_child(update)?;
        }
    } else {
        nix::exec_child(download)?;
    }

    Ok(())
}
