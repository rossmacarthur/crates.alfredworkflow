use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use powerpack::detach;
use powerpack::env;

use crate::logger;

const CRATES_IO_INDEX: &str = "https://github.com/rust-lang/crates.io-index";
pub static FILES: Lazy<Files> = Lazy::new(Files::new);

pub enum IndexStatus {
    Ready,
    Downloading,
    Updating,
}

pub struct Files {
    cache_dir: PathBuf,
    index_dir: PathBuf,
    update_file: PathBuf,
}

impl Files {
    fn new() -> Self {
        let cache_dir = env::workflow_cache().unwrap_or_else(|| {
            let bundle_id = env::workflow_bundle_id()
                .unwrap_or_else(|| String::from("io.macarthur.ross.crates"));
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

    pub fn cache_dir(&self) -> &Path {
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
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::null());
    cmd
}

fn git_clone(url: &str, path: impl AsRef<Path>) -> Result<()> {
    let output = git()
        .args(["clone", "--depth", "1"])
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

fn git_reset(path: impl AsRef<Path>) -> Result<String> {
    let output = git()
        .arg("-C")
        .arg(path.as_ref())
        .args(["reset", "--hard", "origin/HEAD"])
        .output()?;
    if !output.status.success() {
        bail!("failed to run `git reset` command");
    }
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

fn download() -> Result<()> {
    logger::init()?;
    maybe_run(|| {
        let tmp = FILES.index_dir().with_file_name("~crates.io-index");
        fs::remove_dir_all(&tmp).ok();
        git_clone(CRATES_IO_INDEX, &tmp)?;
        fs::rename(&tmp, FILES.index_dir())?;
        fs::File::create(FILES.update_file())?;
        log::info!("downloaded index to ./crates.io-index");
        Ok(())
    })
}

fn update() -> Result<()> {
    logger::init()?;
    maybe_run(|| {
        git_fetch(FILES.index_dir())?;
        let output = git_reset(FILES.index_dir())?;
        fs::File::create(FILES.update_file())?;
        log::info!("updated index ./crates.io-index: {}", output);
        Ok(())
    })
}

fn maybe_run<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if let Some(_guard) = lock_cache_dir()? {
        f()?;
    }
    Ok(())
}

/// Checks that the Crates.io index is okay and returns the path to it.
///
/// This function will spawn a subprocess to clone it if it missing or update it
/// if it is out of date.
pub fn check() -> Result<IndexStatus> {
    let index_dir_exists = FILES.index_dir().exists();

    let index_status = {
        if index_dir_exists {
            match lock_cache_dir()? {
                Some(_guard) => IndexStatus::Ready,
                None => IndexStatus::Updating,
            }
        } else {
            IndexStatus::Downloading
        }
    };

    if index_dir_exists {
        let needs_update = match fs::metadata(FILES.update_file()) {
            Ok(metadata) => {
                let now = SystemTime::now();
                let then = metadata.modified()?;
                now.duration_since(then)? > update_interval()
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => true,
            Err(err) => return Err(err.into()),
        };
        if needs_update {
            detach::spawn(|| {
                if let Err(err) = update() {
                    log::error!("{:#}", err);
                }
            })?;
        }
    } else {
        detach::spawn(|| {
            if let Err(err) = download() {
                log::error!("{:#}", err);
            }
        })?;
    }

    Ok(index_status)
}

fn update_interval() -> Duration {
    let mins = env::var("crates_index_update_interval")
        .and_then(|m| m.parse().ok())
        .unwrap_or(6 * 60);
    Duration::from_secs(mins * 60)
}

fn lock_cache_dir() -> Result<Option<fmutex::Guard>> {
    fmutex::try_lock(FILES.cache_dir())
        .with_context(|| format!("failed to lock `{}`", FILES.cache_dir().display()))
}
