use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::registry::fuzzy;

trait JoinAll {
    fn join_all(&self, all: &[impl AsRef<Path>]) -> PathBuf;
}

impl JoinAll for Path {
    fn join_all(&self, all: &[impl AsRef<Path>]) -> PathBuf {
        let mut p = self.to_owned();
        for segment in all {
            p.push(segment);
        }
        p
    }
}

enum Prefix<'a> {
    /// Just a single path to check.
    One(PathBuf),
    /// A directory to search with no recursion.
    List(PathBuf),
    /// A directory to search and recurse directories.
    Recurse(PathBuf),
    /// A directory to search and recurse once the matching directories.
    RecurseMatching1(PathBuf, &'a str),
    /// A directory to search and recurse twice the matching directories.
    RecurseMatching2(PathBuf, &'a str),
}

fn prefixes(index: PathBuf, query: &str) -> Vec<Prefix> {
    match query.len() {
        0 => vec![],
        1 => {
            vec![
                // ./1/a
                Prefix::One(index.join_all(&["1", query])),
                // ./2/{q*}
                Prefix::List(index.join("2")),
                // ./3/a/{q*}
                Prefix::List(index.join_all(&["3", query])),
                // ./a*/*/{q*}
                Prefix::RecurseMatching2(index, query),
            ]
        }
        2 => {
            vec![
                // ./2/ab
                Prefix::One(index.join_all(&["2", query])),
                // ./3/a/{q*}
                Prefix::List(index.join_all(&["3", &query[0..1]])),
                // ./ab/*/{q*}
                Prefix::Recurse(index.join(query)),
            ]
        }
        3 => {
            vec![
                // ./3/a/abc
                Prefix::One(index.join_all(&["3", &query[0..1], query])),
                // ./ab/c*/{q*}
                Prefix::RecurseMatching1(index.join(&query[0..2]), &query[2..3]),
            ]
        }
        _ => {
            vec![
                // ./ab/cd/{q*}
                Prefix::List(index.join_all(&[&query[0..2], &query[2..4]])),
            ]
        }
    }
}

fn matches(entry: &fs::DirEntry, query: &str) -> bool {
    entry
        .file_name()
        .into_string()
        .map(|f| fuzzy::starts_with(&f, query))
        .unwrap_or(false)
}

fn read_dir(dir: impl AsRef<Path>) -> io::Result<Option<fs::ReadDir>> {
    match fs::read_dir(dir) {
        Ok(r) => Ok(Some(r)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) if err.raw_os_error() == Some(libc::ENOTDIR) => Ok(None),
        Err(err) => Err(err),
    }
}

fn append_files(r: &mut Vec<PathBuf>, dir: impl AsRef<Path>, query: &str) -> io::Result<()> {
    if let Some(rd) = read_dir(dir)? {
        for entry in rd {
            let entry = entry?;
            if matches(&entry, query) {
                r.push(entry.path());
            }
        }
    }
    Ok(())
}

/// Returns a sorted list all the registry files matching the given query.
pub fn all(index: impl Into<PathBuf>, query: &str) -> io::Result<Vec<PathBuf>> {
    let mut r = Vec::new();

    for prefix in prefixes(index.into(), query) {
        match prefix {
            Prefix::One(path) => {
                if path.exists() {
                    r.push(path)
                }
            }
            Prefix::List(path) => {
                append_files(&mut r, path, query)?;
            }
            Prefix::Recurse(path) => {
                if let Some(rd) = read_dir(path)? {
                    for entry in rd {
                        append_files(&mut r, entry?.path(), query)?;
                    }
                }
            }
            Prefix::RecurseMatching1(path, q) => {
                if let Some(rd) = read_dir(path)? {
                    for entry in rd {
                        let entry = entry?;
                        if matches(&entry, q) {
                            append_files(&mut r, entry.path(), query)?;
                        }
                    }
                }
            }
            Prefix::RecurseMatching2(path, q) => {
                if let Some(rd) = read_dir(path)? {
                    for entry in rd {
                        let entry = entry?;
                        if matches(&entry, q) {
                            if let Some(rd) = read_dir(entry.path())? {
                                for entry in rd {
                                    append_files(&mut r, entry?.path(), query)?
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    r.sort_by(fuzzy::cmp);

    Ok(r)
}
