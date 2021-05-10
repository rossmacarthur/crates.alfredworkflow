mod fuzzy;
mod list;

use std::fs;
use std::path::Path;

use anyhow::Result;
use semver::Version;
use serde::Deserialize;

use crate::index::FILES;

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
    Ok(list::all(FILES.index_dir(), query)?.into_iter().filter_map(
        |path| match Package::from_path(&path) {
            Ok(pkg) => Some(pkg),
            Err(err) => {
                eprintln!("Error: {}, {:?}", path.display(), err);
                None
            }
        },
    ))
}
