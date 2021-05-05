use std::cmp::Ordering;
use std::fs;
use std::path::{Component, Path};

use anyhow::Result;
use semver::Version;
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct PackageVersion {
    // Note: field ordering is important for finding the latest non-yanked
    // package version.
    yanked: bool,
    vers: Version,
    name: String,
}

mod fuzzy {
    use super::*;

    fn eq(a: &str, b: &str) -> bool {
        let a = a.as_bytes();
        let b = b.as_bytes();
        if a.len() != b.len() {
            return false;
        }
        a.iter().zip(b.iter()).all(|cmp| match cmp {
            (b'-', b'_') | (b'_', b'-') => true,
            (a, b) => a == b,
        })
    }

    fn starts_with(a: &str, b: &str) -> bool {
        if b.len() > a.len() {
            return false;
        }
        eq(&a[..b.len()], b)
    }

    pub fn matches(query: &str, path: &Path) -> bool {
        let segments: Vec<_> = path
            .components()
            .skip(1)
            .map(Component::as_os_str)
            .map(|s| s.to_str().unwrap())
            .collect();
        if query.is_empty() {
            return false;
        }
        match segments.as_slice() {
            [] => true,
            ["1"] if query.len() <= 1 => true,
            ["2"] if query.len() <= 2 => true,
            ["3"] if query.len() <= 3 => true,
            ["3", a] if query.len() <= 3 && eq(a, &query[0..1]) => true,

            ["1", q] | ["2", q] | ["3", _, q] | [_, _, q] if starts_with(q, query) => true,

            [a] if a.len() == 2 => {
                (query.len() >= 2 && eq(a, &query[0..2]))
                    || (query.len() == 1 && eq(&a[0..1], query))
            }

            [a, b] if a.len() == 2 && b.len() == 2 => {
                (query.len() >= 4 && eq(a, &query[0..2]) && eq(b, &query[2..4]))
                    || (query.len() == 3 && eq(a, &query[0..2]) && eq(&b[0..1], &query[2..3]))
                    || (query.len() == 2 && eq(a, &query[0..2]))
                    || (query.len() == 1 && eq(&a[0..1], query))
            }
            _ => false,
        }
    }

    pub fn cmp(a: &DirEntry, b: &DirEntry) -> Ordering {
        let replace = |e: &DirEntry| e.file_name().to_str().map(|s| s.replace('_', "-"));
        replace(a).cmp(&replace(b))
    }
}

impl Package {
    fn from_path(path: &Path) -> Result<Package> {
        let contents = fs::read_to_string(path)?;
        let PackageVersion { name, vers, .. } = contents
            .lines()
            .map(|line| serde_json::from_str(line))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap();
        Ok(Self {
            name,
            version: vers.to_string(),
        })
    }
}

pub fn walk(query: &str) -> Result<impl Iterator<Item = Package> + '_> {
    let index = crate::index::cache_dir()?;

    let paths: Vec<_> = WalkDir::new(&index)
        .sort_by(fuzzy::cmp)
        .into_iter()
        .filter_entry(move |entry| {
            fuzzy::matches(query, entry.path().strip_prefix(&index).unwrap())
        })
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(err) => {
                eprintln!("Error: {:?}", err);
                None
            }
        })
        .filter(|entry| !entry.path().is_dir())
        .map(|entry| entry.into_path())
        .collect();

    Ok(paths
        .into_iter()
        .filter_map(|path| match Package::from_path(&path) {
            Ok(pkg) => Some(pkg),
            Err(err) => {
                eprintln!("Error: {}, {:?}", path.display(), err);
                None
            }
        }))
}

#[test]
fn fuzzy_matches() {
    let test_cases = &[
        // query length 0
        // --------------
        ("", "registry/1", false),
        ("", "registry/1/a", false),
        ("", "registry/1/x", false),
        ("", "registry/2", false),
        ("", "registry/2/ab", false),
        ("", "registry/2/ax", false),
        ("", "registry/2/xb", false),
        ("", "registry/3", false),
        ("", "registry/3/a", false),
        ("", "registry/3/x", false),
        ("", "registry/3/a/abc", false),
        ("", "registry/3/a/abx", false),
        ("", "registry/3/x/xbc", false),
        ("", "registry/ab", false),
        ("", "registry/xb", false),
        ("", "registry/ab/cd", false),
        ("", "registry/xb/cd", false),
        ("", "registry/ab/xy", false),
        ("", "registry/ab/cd/abcd", false),
        ("", "registry/ab/cd/abcde", false),
        // query length 1
        // --------------
        ("a", "registry/1", true),
        ("a", "registry/1/a", true),
        ("a", "registry/1/x", false),
        ("a", "registry/2", true),
        ("a", "registry/2/ab", true),
        ("a", "registry/2/ax", true),
        ("a", "registry/2/xb", false),
        ("a", "registry/3", true),
        ("a", "registry/3/a", true),
        ("a", "registry/3/x", false),
        ("a", "registry/3/a/abc", true),
        ("a", "registry/3/a/abx", true),
        ("a", "registry/3/x/xbc", false),
        ("a", "registry/ab", true),
        ("a", "registry/xb", false),
        ("a", "registry/ab/cd", true),
        ("a", "registry/xb/cd", false),
        ("a", "registry/ab/xy", true),
        ("a", "registry/ab/cd/abcd", true),
        ("a", "registry/ab/cd/abcde", true),
        // query length 2
        // --------------
        ("ab", "registry/1", false),
        ("ab", "registry/1/a", false),
        ("ab", "registry/1/x", false),
        ("ab", "registry/2", true),
        ("ab", "registry/2/ab", true),
        ("ab", "registry/2/ax", false),
        ("ab", "registry/2/xb", false),
        ("ab", "registry/3", true),
        ("ab", "registry/3/a", true),
        ("ab", "registry/3/x", false),
        ("ab", "registry/3/a/abc", true),
        ("ab", "registry/3/a/abx", true),
        ("ab", "registry/3/x/xbc", false),
        ("ab", "registry/ab", true),
        ("ab", "registry/xb", false),
        ("ab", "registry/ab/cd", true),
        ("ab", "registry/xb/cd", false),
        ("ab", "registry/ab/xy", true),
        ("ab", "registry/ab/cd/abcd", true),
        ("ab", "registry/ab/cd/abcde", true),
        // query length 3
        // --------------
        ("abc", "registry/1", false),
        ("abc", "registry/1/a", false),
        ("abc", "registry/1/x", false),
        ("abc", "registry/2", false),
        ("abc", "registry/2/ab", false),
        ("abc", "registry/2/ax", false),
        ("abc", "registry/2/xb", false),
        ("abc", "registry/3", true),
        ("abc", "registry/3/a", true),
        ("abc", "registry/3/x", false),
        ("abc", "registry/3/a/abc", true),
        ("abc", "registry/3/a/abx", false),
        ("abc", "registry/3/x/xbc", false),
        ("abc", "registry/ab", true),
        ("abc", "registry/xb", false),
        ("abc", "registry/ab/cd", true),
        ("abc", "registry/xb/cd", false),
        ("abc", "registry/ab/xy", false),
        ("abc", "registry/ab/cd/abcd", true),
        ("abc", "registry/ab/cd/abcde", true),
        // query length 4
        // --------------
        ("abcd", "registry/1", false),
        ("abcd", "registry/1/a", false),
        ("abcd", "registry/1/x", false),
        ("abcd", "registry/2", false),
        ("abcd", "registry/2/ab", false),
        ("abcd", "registry/2/ax", false),
        ("abcd", "registry/2/xb", false),
        ("abcd", "registry/3", false),
        ("abcd", "registry/3/a", false),
        ("abcd", "registry/3/x", false),
        ("abcd", "registry/3/a/abc", false),
        ("abcd", "registry/3/a/abx", false),
        ("abcd", "registry/3/x/xbc", false),
        ("abcd", "registry/ab", true),
        ("abcd", "registry/xb", false),
        ("abcd", "registry/ab/cd", true),
        ("abcd", "registry/xb/cd", false),
        ("abcd", "registry/ab/xy", false),
        ("abcd", "registry/ab/cd/abcd", true),
        ("abcd", "registry/ab/cd/abcde", true),
        // query hypens/underscores
        // ------------------------
        ("-", "registry/1", true),
        ("-", "registry/1/-", true),
        ("-", "registry/1/_", true),
        ("-", "registry/1/a", false),
        ("-b", "registry/2", true),
        ("-b", "registry/2/-b", true),
        ("-b", "registry/2/_b", true),
        ("-b", "registry/2/ab", false),
        ("-bc", "registry/3", true),
        ("-bc", "registry/3/-", true),
        ("-bc", "registry/3/_", true),
        ("-bc", "registry/3/a", false),
        ("-bcd", "registry/-b", true),
        ("-bcd", "registry/_b", true),
        ("-bcd", "registry/-b/cd", true),
        ("-bcd", "registry/_b/cd", true),
        ("-bcd", "registry/-b/cd/-bcd", true),
        ("-bcd", "registry/_b/cd/_bcd", true),
    ];

    for (query, path, result) in test_cases {
        assert_eq!(
            fuzzy::matches(query, Path::new(path)),
            *result,
            "query = {}, path = {}",
            query,
            path
        );
    }
}
