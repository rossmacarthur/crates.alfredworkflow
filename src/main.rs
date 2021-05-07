mod index;
mod registry;

use std::env;
use std::iter;

use anyhow::Result;
use powerpack::{Icon, Item, ModifierData, ModifierKey};

fn empty() -> Item<'static> {
    Item::new("Search for crates")
        .subtitle("Open Crates.io →")
        .arg("https://crates.io")
        .icon(Icon::new("icon.png"))
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

fn default(query: &str) -> Item<'static> {
    Item::new(format!("Search for '{}'", query))
        .subtitle(format!("Search Crates.io for '{}' →", query))
        .arg(format!("https://crates.io/search?q={}", query))
        .icon(Icon::new("icon.png"))
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

fn to_item(pkg: registry::Package) -> Item<'static> {
    Item::new(format!("{} v{}", pkg.name, pkg.version))
        .subtitle("Open in Crates.io →")
        .arg(format!("https://crates.io/crates/{}", pkg.name))
        .icon(Icon::new("icon.png"))
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

fn output(query: &str) -> Result<()> {
    index::check()?;
    powerpack::output(
        registry::walk(query)?
            .take(10)
            .map(to_item)
            .chain(iter::once(default(query))),
    )?;
    Ok(())
}

fn main() -> Result<()> {
    match env::args().nth(1).as_deref().map(str::trim) {
        None | Some("") => powerpack::output(iter::once(empty()))?,
        Some(query) => output(query)?,
    }
    Ok(())
}
