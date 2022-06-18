mod fuzzy;
mod list;

use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use anyhow::Result;
use semver::Version;
use serde::Deserialize;

use crate::index::FILES;
use crate::Package;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct PackageVersion {
    name: String,
    vers: Version,
    yanked: bool,
}

pub fn walk(query: &str) -> Result<impl Iterator<Item = Package> + '_> {
    let index = list::all(FILES.index_dir(), query)?
        .into_iter()
        .filter_map(|path| match make_package(&path) {
            Ok(pkg) => Some(pkg),
            Err(err) => {
                eprintln!("Error: {}, {:?}", path.display(), err);
                None
            }
        });

    Ok(index)
}

fn make_package(path: &Path) -> Result<Package> {
    let contents = fs::read_to_string(path)?;
    let PackageVersion { name, vers, .. } = contents
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max_by(cmp)
        .unwrap();
    Ok(Package::Registry {
        name,
        version: vers.to_string(),
    })
}

/// Order by unyanked then version number.
fn cmp(a: &PackageVersion, b: &PackageVersion) -> Ordering {
    (!a.yanked, &a.vers).cmp(&(!b.yanked, &b.vers))
}
