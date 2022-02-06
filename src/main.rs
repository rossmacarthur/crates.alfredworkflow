mod index;
mod registry;

use std::env;
use std::iter;

use anyhow::Result;
use either::Either;
use powerpack::{Item, ModifierData, ModifierKey};

/// Returns an Alfred item for when no query has been typed yet.
fn empty() -> Item<'static> {
    Item::new("Search for crates")
        .subtitle("Open Crates.io →")
        .arg("https://crates.io")
        .modifier(
            ModifierKey::Option,
            ModifierData::new()
                .subtitle("Open Lib.rs →")
                .arg("https://lib.rs"),
        )
        .modifier(
            ModifierKey::Shift,
            ModifierData::new()
                .subtitle("Open Docs.rs →")
                .arg("https://docs.rs"),
        )
}

/// Returns an Alfred item for when the query doesn't match any crates.
fn default(query: &str) -> Item<'static> {
    Item::new(format!("Search for '{}'", query))
        .subtitle(format!("Search Crates.io for '{}' →", query))
        .arg(format!("https://crates.io/search?q={}", query))
        .modifier(
            ModifierKey::Option,
            ModifierData::new()
                .subtitle(format!("Search Lib.rs for '{}' →", query))
                .arg(format!("https://lib.rs/search?q={}", query)),
        )
        .modifier(
            ModifierKey::Shift,
            ModifierData::new()
                .subtitle(format!("Search Docs.rs for '{}' →", query))
                .arg(format!("https://docs.rs/releases/search?query={}", query)),
        )
}

/// Converts a registry package to an Alfred item.
fn to_item(pkg: registry::Package) -> Item<'static> {
    Item::new(format!("{} v{}", pkg.name, pkg.version))
        .subtitle("Open in Crates.io →")
        .arg(format!("https://crates.io/crates/{}", pkg.name))
        .modifier(
            ModifierKey::Option,
            ModifierData::new()
                .subtitle("Open in Lib.rs →")
                .arg(format!("https://lib.rs/crates/{}", pkg.name)),
        )
        .modifier(
            ModifierKey::Shift,
            ModifierData::new()
                .subtitle("Open in Docs.rs →")
                .arg(format!("https://docs.rs/{}", pkg.name)),
        )
}

fn main() -> Result<()> {
    let arg = env::args()
        .nth(1)
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase);
    let items = match arg.as_deref() {
        None | Some("") => Either::Left(iter::once(empty())),
        Some(query) => {
            index::check()?;
            Either::Right(
                registry::walk(query)?
                    .take(10)
                    .map(to_item)
                    .chain(iter::once(default(query))),
            )
        }
    };
    powerpack::output(items)?;
    Ok(())
}
