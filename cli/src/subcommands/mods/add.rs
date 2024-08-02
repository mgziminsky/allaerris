use anyhow::Result;
use relibium::{
    config::{profile::ProfileData, Mod, VersionedProject},
    Client,
};
use yansi::Paint;

use crate::tui::{mod_single_line, CROSS_RED, TICK_GREEN, TICK_YELLOW};

pub async fn add(client: Client, profile: &mut ProfileData, ids: Vec<String>, exclude: bool) -> Result<()> {
    eprintln!("Fetching mod information...");
    let mods = if ids.len() == 1 {
        let m = client.get_mod(&ids[0]).await?;
        println!("{:^4}{}", TICK_GREEN, m.name.bold());
        vec![m]
    } else {
        let ids = ids.iter().map(|id| id as _).collect::<Vec<_>>();
        let mods = client.get_mods(&ids).await?;
        println!("{:^4}{} mods found", TICK_GREEN, mods.len());
        mods
    }
    .into_iter()
    .map(Mod::from) // From schema to config Mod
    .map(|mut m| {
        m.exclude = exclude;
        m
    })
    .collect::<Vec<_>>();

    for res in profile.add_mods(&mods) {
        match res {
            Ok(m) | Err(m) => println!("{:^4}{}", if res.is_ok() { TICK_GREEN } else { TICK_YELLOW }, mod_single_line(m)),
        }
    }
    // Show not found
    ids.into_iter()
        .filter(|id| !mods.iter().any(|m| &m.slug == id || m.project() == id))
        .for_each(|id| println!("{:^4}{} — Not Found", CROSS_RED, id.italic().bold()));

    Ok(())
}
