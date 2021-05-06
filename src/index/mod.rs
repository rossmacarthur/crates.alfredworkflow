mod mutex;
mod nix;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Result};
use once_cell::sync::Lazy;

const UPDATE_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);
const CRATE_ID: &str = "io.macarthur.ross.crates";
const CRATES_IO_INDEX: &str = "https://github.com/rust-lang/crates.io-index";

static HOME_DIR: Lazy<PathBuf> = Lazy::new(|| home::home_dir().unwrap());
pub static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| HOME_DIR.join("Library/Caches").join(CRATE_ID));
static INDEX_DIR: Lazy<PathBuf> = Lazy::new(|| CACHE_DIR.join("crates.io-index"));
static UPDATE_FILE: Lazy<PathBuf> = Lazy::new(|| INDEX_DIR.join(".last-modified"));

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
    let _mutex = mutex::acquire(&*CACHE_DIR)?;
    git_clone(CRATES_IO_INDEX, &*INDEX_DIR)?;
    fs::File::create(&*UPDATE_FILE)?;
    Ok(())
}

fn update() -> Result<()> {
    let _mutex = mutex::acquire(&*CACHE_DIR)?;
    git_fetch(&*INDEX_DIR)?;
    git_reset(&*INDEX_DIR)?;
    fs::File::create(&*UPDATE_FILE)?;
    Ok(())
}

/// Checks that the Crates.io index is okay and returns the path to it.
///
/// This function will spawn a subprocess to clone it if it missing or update it
/// if it is out of date.
pub fn check() -> Result<()> {
    if INDEX_DIR.exists() {
        let needs_update = match fs::metadata(&*UPDATE_FILE) {
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
