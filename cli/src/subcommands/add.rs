use anyhow::Result;
use colored::Colorize;
use relibium::{
    config::{profile::ProfileData, Mod, VersionedProject},
    Client,
};

use crate::tui::{mod_single_line, CROSS_RED, TICK_GREEN, TICK_YELLOW};

pub async fn add(client: Client, profile: &mut ProfileData, ids: Vec<String>, exclude: bool) -> Result<()> {
    eprint!("Fetching mod information...");
    let mods = if ids.len() == 1 {
        let m = client.get_mod(&ids[0]).await?;
        eprintln!();
        println!("{}\t{}", *TICK_GREEN, m.name.bold());
        vec![m]
    } else {
        let ids = ids.iter().map(|id| id as _).collect::<Vec<_>>();
        let mods = client.get_mods(&ids).await?;
        eprintln!("{}", *TICK_GREEN);
        mods
    }
    .into_iter()
    .map(Mod::from) // From schema to config Mod
    .map(|mut m| {
        m.exclude = exclude;
        m
    })
    .collect::<Vec<_>>();

    let added = profile.add_mods(mods.iter());
    // Show already added
    added.iter().filter_map(|r| r.err()).for_each(|m| {
        println!("{}\t{}", *TICK_YELLOW, mod_single_line(m));
    });
    // Show newly added
    added.iter().filter_map(|r| r.ok()).for_each(|m| {
        println!("{}\t{}", *TICK_GREEN, mod_single_line(m));
    });
    // Show not found
    ids.into_iter()
        .filter(|id| !mods.iter().any(|m| &m.slug == id || m.project() == id))
        .for_each(|id| println!("{}\t{} — Not Found", *CROSS_RED, id.italic().bold()));

    Ok(())
}
