mod registry;

use std::env;
use std::iter;

use anyhow::Result;

fn to_item(pkg: registry::Package) -> powerpack::Item<'static> {
    powerpack::Item::new(format!("{} v{}", pkg.name, pkg.vers))
        .subtitle("Open in Crates.io →")
        .arg(format!("https://crates.io/crates/{}", pkg.name))
        .icon(powerpack::Icon::new("icon.png"))
}

fn output(pkgs: impl Iterator<Item = registry::Package>) -> Result<()> {
    Ok(powerpack::output(pkgs.map(to_item))?)
}

fn main() -> Result<()> {
    match env::args().nth(1).map(|q| q.trim().to_string()) {
        None => output(iter::empty()),
        Some(query) => output(registry::walk(&query)?.take(50)),
    }
}
