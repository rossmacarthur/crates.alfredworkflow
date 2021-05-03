use std::env;
use std::iter;

use anyhow::Result;

fn main() -> Result<()> {
    // Alfred passes in a single argument for the user query.
    let query = env::args().nth(1);

    // Now we create an item to show in the Alfred drop down.
    let item = powerpack::Item::new("Hello world!")
        .subtitle(format!("Your query was '{:?}'", query))
        .icon(powerpack::Icon::from_file_type("public.script"));

    // Output the items to Alfred!
    powerpack::output(iter::once(item))?;

    Ok(())
}
