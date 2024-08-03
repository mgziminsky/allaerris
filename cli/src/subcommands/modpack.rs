use anyhow::{anyhow, bail, Ok, Result};
use dialoguer::Confirm;
use relibium::{
    config::{profile::ProfileData, Modpack, Profile},
    Client,
};
use yansi::Paint;

use crate::{
    cli::ModpackSubcommand,
    tui::{mod_single_line, CROSS_RED, THEME, TICK_GREEN},
};

const MSG_NO_PACK: &str = "No modpack on active profile";

pub async fn process(subcommand: ModpackSubcommand, profile: &mut Profile, client: Client) -> Result<()> {
    match subcommand {
        ModpackSubcommand::Info => {
            let pack = &profile.data().await?.modpack;
            if let Some(ref pack) = pack {
                print_pack(pack);
            }
        },
        ModpackSubcommand::Add { id, install_overrides } => {
            add(id, profile.data_mut().await?, install_overrides, &client).await?;
        },
        ModpackSubcommand::Remove { force } => {
            remove(profile.data_mut().await?, force)?;
        },
        ModpackSubcommand::Configure { install_overrides } => {
            let mp = profile.data_mut().await?.modpack.as_mut().ok_or_else(|| anyhow!(MSG_NO_PACK))?;
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
        if pack.install_overrides { TICK_GREEN } else { CROSS_RED }
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

async fn add(id: String, data: &mut ProfileData, install_overrides: Option<bool>, client: &Client) -> Result<()> {
    if data.modpack.is_some()
        && !Confirm::with_theme(&*THEME)
            .default(false)
            .with_prompt("Active profile already has a modpack set. Do you want to replace it?")
            .interact()?
    {
        bail!("Modpack update cancelled");
    }

    let pack = client.get_modpack(&id).await?;
    let install_overrides = prompt_overrides(install_overrides, true)?;
    data.modpack.replace(Modpack::new(pack, install_overrides));

    Ok(())
}

fn remove(data: &mut ProfileData, force: bool) -> Result<(), anyhow::Error> {
    let Some(ref modpack) = data.modpack else {
        bail!(MSG_NO_PACK);
    };
    if force
        || Confirm::with_theme(&*THEME)
            .default(true)
            .with_prompt(format!("Remove modpack `{}` from active profile?", mod_single_line(modpack)))
            .interact()?
    {
        data.modpack = None;
    }
    Ok(())
}
