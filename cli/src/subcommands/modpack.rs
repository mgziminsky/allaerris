use anyhow::{anyhow, bail, Ok, Result};
use colored::Colorize;
use dialoguer::Confirm;
use relibium::{
    config::{Modpack, Profile},
    Client, Config,
};

use crate::{
    cli::ModpackSubCommand,
    helpers::get_active_profile,
    tui::{mod_single_line, CROSS_RED, THEME, TICK_GREEN},
};

const MSG_NO_PACK: &str = "No modpack on active profile";

pub async fn process(subcommand: ModpackSubCommand, config: &mut Config, client: Client) -> Result<()> {
    match subcommand {
        ModpackSubCommand::Info => {
            let pack = &get_active_profile(config)?.data().await?.modpack;
            if let Some(ref pack) = pack {
                print_pack(pack);
            }
        },
        ModpackSubCommand::Add { id, install_overrides } => {
            add(id, get_active_profile(config)?, install_overrides, &client).await?;
        },
        ModpackSubCommand::Remove { force } => {
            let profile = get_active_profile(config)?.data_mut().await?;
            if let Some(ref modpack) = profile.modpack {
                if force
                    || Confirm::with_theme(&*THEME)
                        .default(true)
                        .with_prompt(format!("Remove modpack `{}` from active profile?", mod_single_line(modpack)))
                        .interact()?
                {
                    profile.modpack = None;
                }
            } else {
                bail!(MSG_NO_PACK)
            }
        },
        ModpackSubCommand::Configure { install_overrides } => {
            let mp = get_active_profile(config)?
                .data_mut()
                .await?
                .modpack
                .as_mut()
                .ok_or_else(|| anyhow!(MSG_NO_PACK))?;
            mp.install_overrides = prompt_overrides(install_overrides, mp.install_overrides)?;
        },
    }
    Ok(())
}

fn print_pack(pack: &Modpack) {
    println!(
        "\
{}
Slug:  {}
Install Overrides: {}
",
        mod_single_line(pack),
        pack.slug.italic(),
        if pack.install_overrides { &*TICK_GREEN } else { &*CROSS_RED }
    );
}

fn prompt_overrides(initial: Option<bool>, default: bool) -> Result<bool> {
    let install_overrides = match initial {
        Some(overrides) => overrides,
        None => Confirm::with_theme(&*THEME)
            .default(default)
            .with_prompt("Should overrides be installed?")
            .interact()?,
    };
    if install_overrides {
        println!(
            "{}",
            "WARNING: Files in your profile directory may be overwritten by modpack overrides"
                .yellow()
                .bold()
        );
    }
    Ok(install_overrides)
}

async fn add(id: String, profile: &mut Profile, install_overrides: Option<bool>, client: &Client) -> Result<()> {
    let profile = profile.data_mut().await?;
    if profile.modpack.is_some()
        && !Confirm::with_theme(&*THEME)
            .default(false)
            .with_prompt("Active profile already has a modpack set. Do you want to replace it?")
            .interact()?
    {
        return Ok(());
    }

    let pack = client.get_modpack(&id).await?;
    let install_overrides = prompt_overrides(install_overrides, true)?;
    profile.modpack.replace(Modpack::new(pack, install_overrides));

    Ok(())
}
