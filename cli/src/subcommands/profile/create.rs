use std::path::PathBuf;

use anyhow::{Context, Ok, Result, bail};
use dialoguer::{Confirm, Input};
use ferrallay::{
    Config, DEFAULT_MINECRAFT_DIR,
    checked_types::PathAbsolute,
    config::{ModLoader, Profile, profile::ProfileData},
};
use yansi::Paint;

use super::helpers::{pick_minecraft_version, pick_mod_loader};
use crate::{
    file_picker::pick_folder,
    tui::{THEME, TICK_GREEN},
};

pub async fn create(
    config: &mut Config,
    game_version: Option<String>,
    loader: Option<ModLoader>,
    name: Option<String>,
    path: Option<PathBuf>,
    server: bool,
) -> Result<()> {
    let path = path.map_or_else(
        || {
            println!(
                "The default profile directory is `{}`",
                DEFAULT_MINECRAFT_DIR.display().bold().italic()
            );
            if config.profile(&*DEFAULT_MINECRAFT_DIR).is_ok()
                || Confirm::with_theme(&*THEME)
                    .with_prompt("Would you like to specify a custom profile directory?")
                    .interact()?
            {
                Ok(pick_folder(&*DEFAULT_MINECRAFT_DIR, "Pick a profile directory"))
            } else {
                Ok(DEFAULT_MINECRAFT_DIR.to_path_buf())
            }
        },
        Ok,
    )?;
    let path = PathAbsolute::new(path)?;
    if config.profile(&path).is_ok() {
        bail!("Config already contains a profile at the path `{}`", path.display().bold().italic())
    }
    if ProfileData::file_path(&path).exists() {
        bail!(
            "A profile config file already exists at `{}`\n# Use import command instead",
            path.display().bold().italic()
        )
    }
    println!("{} {} = {}", TICK_GREEN, "Profile Directory".bold(), path.display().green());

    let name = name.map_or_else(
        || loop {
            let name: String = Input::with_theme(&*THEME)
                .with_prompt("What should this profile be called?")
                .interact_text()?;
            if !name.trim().is_empty() {
                break Ok(name);
            }
        },
        Ok,
    )?;

    let loader = loader.map_or_else(|| pick_mod_loader(None), Ok)?;

    let game_version = match game_version {
        Some(gv) => gv,
        None => pick_minecraft_version(None).await?,
    };

    let profile = Profile::with_data(name, path.clone(), ProfileData {
        game_version,
        loader,
        mods: vec![],
        modpack: None,
        is_server: server,
    });
    config
        .add_profile(profile)
        .expect("shouldn't fail to add profile since conditions were checked before");
    let _ = config
        .set_active(path)
        .context("Failed to switch to newly created profile")
        .inspect_err(|e| eprintln!("{:?}", e.yellow()))
        .inspect(|()| println!("The newly created profile is now active"));

    Ok(())
}
