use anyhow::Result;
use relibium::{
    config::{profile::ProfileData, Mod, VersionedProject},
    Client,
};
use yansi::Paint;

use crate::tui::{mod_single_line, CROSS_RED, TICK_GREEN, TICK_YELLOW};

/// Add mods with `ids` to `profile` returning the number of added/updated mods,
/// not counting existing and unchanged mods
pub async fn add(client: &Client, profile: &mut ProfileData, ids: Vec<String>, exclude: bool) -> Result<usize> {
    eprintln!("Fetching mod information...");
    let mods = if ids.len() == 1 {
        let m = client.get_mod(&ids[0]).await?;
        println!("{} {}", TICK_GREEN, m.name.bold());
        vec![m]
    } else {
        let ids = ids.iter().map(|id| id as _).collect::<Vec<_>>();
        let mods = client.get_mods(&ids).await?;
        println!("{} {}/{} mods found", TICK_GREEN, mods.len().bright_blue(), ids.len().blue());
        mods
    }
    .into_iter()
    .map(Mod::from) // From schema to config Mod
    .map(|mut m| {
        m.exclude = exclude;
        m
    })
    .collect::<Vec<_>>();

    let mut new = 0;
    for res in profile.add_mods(&mods) {
        match res {
            Ok(m) | Err(m) => {
                res.is_ok().then(|| new += 1);
                println!("{} {}", if res.is_ok() { TICK_GREEN } else { TICK_YELLOW }, mod_single_line(m));
            },
        }
    }
    // Show not found
    ids.into_iter()
        .filter(|id| !mods.iter().any(|m| &m.slug == id || m.project() == id))
        .for_each(|id| println!("{} {} — Not Found", CROSS_RED, id.italic().bold()));

    Ok(new)
}
