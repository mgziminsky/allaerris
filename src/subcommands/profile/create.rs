use anyhow::{bail, Ok, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use relibium::{
    config::{profile::ProfileData, ModLoader, Profile},
    Client, Config, DEFAULT_MINECRAFT_DIR,
};
use std::path::PathBuf;

use crate::{
    file_picker::pick_folder,
    subcommands::profile::{check_profile_path, pick_mod_loader},
    tui::{THEME, TICK_GREEN},
};

pub async fn create(
    client: &Client,
    config: &mut Config,
    game_version: Option<String>,
    loader: Option<ModLoader>,
    name: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let path = path.map_or_else(
        || {
            println!(
                "The default mods directory is {}",
                DEFAULT_MINECRAFT_DIR.display()
            );
            if Confirm::with_theme(&*THEME)
                .with_prompt("Would you like to specify a custom profile directory?")
                .interact()?
            {
                pick_folder(DEFAULT_MINECRAFT_DIR.as_path(), "Pick a profile directory")
            } else {
                Ok(DEFAULT_MINECRAFT_DIR.clone())
            }
        },
        Ok,
    )?;
    check_profile_path(&path)?;
    if config.profile(&path).is_ok() {
        bail!(
            "Config already contains a profile at the path `{}`",
            path.display()
        )
    }
    println!(
        "{} {} · {}",
        *TICK_GREEN,
        "Profile Directory".bold(),
        path.display().to_string().green()
    );

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

    let game_version = if let Some(gv) = game_version {
        gv
    } else {
        let mut versions: Vec<_> = client
            .get_game_versions()
            .await?
            .into_iter()
            .map(|v| v.version)
            .collect();

        let selected_version = Select::with_theme(&*THEME)
            .with_prompt("Which version of Minecraft should this profile use?")
            .items(&versions)
            .interact()?;

        versions.swap_remove(selected_version)
    };

    let profile = Profile::with_data(
        name,
        path,
        ProfileData {
            game_version,
            loader,
            mods: vec![],
            modpack: None,
        },
    )?;
    config.add_profile(profile).expect("shouldn't fail to add profile since conditions were checked before");

    Ok(())
}
