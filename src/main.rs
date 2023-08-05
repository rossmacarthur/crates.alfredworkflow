mod index;
mod logger;
mod registry;

use std::env;
use std::iter;

use crate::index::IndexStatus;
use anyhow::Result;
use either::Either;
use powerpack::{Item, Key, Modifier};

#[derive(Debug)]
pub enum Package {
    Builtin { name: &'static str },
    Registry { name: String, version: String },
}

fn builtins(query: &str) -> impl Iterator<Item = Package> + '_ {
    ["alloc", "core", "std"]
        .iter()
        .filter(move |name| name.starts_with(query))
        .map(|name| Package::Builtin { name })
}

/// Returns an Alfred item for when no query has been typed yet.
fn empty() -> Item {
    Item::new("Search for crates")
        .subtitle("Open Crates.io →")
        .arg("https://crates.io")
        .modifier(
            Modifier::new(Key::Option)
                .subtitle("Open Lib.rs →")
                .arg("https://lib.rs"),
        )
        .modifier(
            Modifier::new(Key::Shift)
                .subtitle("Open Docs.rs →")
                .arg("https://docs.rs"),
        )
}

/// Returns an Alfred item for when the query doesn't match any crates.
fn default(query: &str) -> Item {
    Item::new(format!("Search for '{}'", query))
        .subtitle(format!("Search Crates.io for '{}' →", query))
        .arg(format!("https://crates.io/search?q={}", query))
        .modifier(
            Modifier::new(Key::Option)
                .subtitle(format!("Search Lib.rs for '{}' →", query))
                .arg(format!("https://lib.rs/search?q={}", query)),
        )
        .modifier(
            Modifier::new(Key::Shift)
                .subtitle(format!("Search Docs.rs for '{}' →", query))
                .arg(format!("https://docs.rs/releases/search?query={}", query)),
        )
}

/// Converts a registry package to an Alfred item.
fn to_item(pkg: Package) -> Item {
    match pkg {
        Package::Builtin { name } => Item::new(name)
            .subtitle("Open official documentation (stable) →")
            .arg(format!("https://doc.rust-lang.org/stable/{}/", name))
            .modifier(
                Modifier::new(Key::Option)
                    .subtitle("Open official documentation (nightly) →")
                    .arg(format!("https://doc.rust-lang.org/nightly/{}/", name)),
            ),
        Package::Registry { name, version } => Item::new(format!("{} v{}", name, version))
            .subtitle("Open in Crates.io →")
            .arg(format!("https://crates.io/crates/{}", name))
            .modifier(
                Modifier::new(Key::Option)
                    .subtitle("Open in Lib.rs →")
                    .arg(format!("https://lib.rs/crates/{}", name)),
            )
            .modifier(
                Modifier::new(Key::Shift)
                    .subtitle("Open in Docs.rs →")
                    .arg(format!("https://docs.rs/{}", name)),
            ),
    }
}

fn append_index_status(items: &mut Vec<Item>, status: IndexStatus) {
    match status {
        IndexStatus::Ready => {}
        IndexStatus::Downloading => items.push(
            Item::new("Downloading index...")
                .subtitle("The local Crates.io index is being downloaded. This may take a while."),
        ),
        IndexStatus::Updating => items.push(
            Item::new("Updating index...").subtitle("The local Crates.io index is being updated"),
        ),
    };
}

fn main() -> Result<()> {
    let arg = env::args()
        .nth(1)
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase);

    let index_status = index::check()?;

    let mut items = Vec::from_iter(match arg.as_deref() {
        None | Some("") => Either::Left(iter::once(empty())),
        Some(query) => {
            let iter = builtins(query)
                .chain(registry::walk(query)?.take(10))
                .map(to_item)
                .chain(iter::once(default(query)));
            Either::Right(iter)
        }
    });

    append_index_status(&mut items, index_status);

    powerpack::output(items)?;

    Ok(())
}
