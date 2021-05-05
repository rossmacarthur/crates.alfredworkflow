mod mutex;
mod nix;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context as _, Result};

const UPDATE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const CRATE_ID: &str = "io.macarthur.ross.crates";
const CRATES_IO_INDEX: &str = "https://github.com/rust-lang/crates.io-index";

struct Dirs {
    cache_dir: PathBuf,
    index_dir: PathBuf,
    update_file: PathBuf,
}

impl Dirs {
    pub fn new() -> Result<Self> {
        let cache_dir = cache_dir()?;
        let index_dir = cache_dir.join("crates.io-index");
        let update_file = index_dir.join(".last-modified");

        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            index_dir,
            update_file,
        })
    }
}

pub fn cache_dir() -> Result<PathBuf> {
    let mut cache_dir =
        home::home_dir().context("failed to determine the current user's home directory")?;
    cache_dir.push("Library/Caches");
    cache_dir.push(CRATE_ID);
    Ok(cache_dir)
}

fn git_clone(url: &str, path: impl AsRef<Path>) -> Result<()> {
    let output = process::Command::new("git")
        .args(&["clone", "--depth", "1"])
        .arg(url)
        .arg(path.as_ref())
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .output()?;
    if !output.status.success() {
        bail!("failed to run `git clone` command");
    }
    Ok(())
}

fn git_pull(path: impl AsRef<Path>) -> Result<()> {
    let output = process::Command::new("git")
        .arg("-C")
        .arg(path.as_ref())
        .arg("pull")
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .output()?;
    if !output.status.success() {
        bail!("failed to run `git clone` command");
    }
    Ok(())
}

fn download(dirs: &Dirs) -> Result<()> {
    let Dirs {
        cache_dir,
        index_dir,
        update_file,
    } = dirs;
    let _mutex = mutex::acquire(cache_dir)?;
    git_clone(CRATES_IO_INDEX, index_dir)?;
    fs::File::create(update_file)?;
    Ok(())
}

fn update(dirs: &Dirs) -> Result<()> {
    let Dirs {
        cache_dir,
        index_dir,
        update_file,
    } = dirs;
    let _mutex = mutex::acquire(cache_dir)?;
    git_pull(&index_dir)?;
    fs::File::create(&update_file)?;
    Ok(())
}

/// Checks that the Crates.io index is okay and returns the path to it.
///
/// This function will spawn a subprocess to clone it if it missing or update it
/// if it is out of date.
pub fn check() -> Result<()> {
    let dirs = Dirs::new()?;

    if dirs.index_dir.exists() {
        let needs_update = match fs::metadata(&dirs.update_file) {
            Ok(metadata) => {
                let now = SystemTime::now();
                let then = metadata.modified()?;
                now.duration_since(then)? > UPDATE_INTERVAL
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => true,
            Err(err) => return Err(err.into()),
        };
        if needs_update {
            nix::exec_child(|| update(&dirs))?;
        }
    } else {
        nix::exec_child(|| download(&dirs))?;
    }

    Ok(())
}
