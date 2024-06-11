mod configure;
mod create;
mod delete;
pub use configure::configure;
pub use create::create;
pub use delete::delete;

use anyhow::{bail, Result};
use dialoguer::Select;
use relibium::config::{ModLoader, Profile};
use std::{ops::Deref, path::Path};

use crate::tui::{fmt_profile_simple, THEME};

pub fn pick_mod_loader(default: Option<ModLoader>) -> Result<ModLoader> {
    let mut picker = Select::with_theme(&*THEME)
        .with_prompt("Select the mod loader to use:")
        .items(&[
            "Fabric",
            "NeoForge",
            "Forge",
            "Quilt",
            "LiteLoader",
            "Cauldron",
        ]);

    if let Some(default) = default {
        picker = picker.default(match default {
            ModLoader::Fabric => 0,
            ModLoader::NeoForge => 1,
            ModLoader::Forge => 2,
            ModLoader::Quilt => 3,
            ModLoader::LiteLoader => 4,
            ModLoader::Cauldron => 5,
            _ => !0,
        });
    }

    picker
        .interact()
        .map(|i| match i {
            0 => ModLoader::Fabric,
            1 => ModLoader::NeoForge,
            2 => ModLoader::Forge,
            3 => ModLoader::Quilt,
            4 => ModLoader::LiteLoader,
            5 => ModLoader::Cauldron,
            _ => unreachable!(),
        })
        .map_err(Into::into)
}

pub async fn pick_minecraft_version() -> Result<String> {
    todo!("Move code from delete to here")
}

pub fn pick_profile<'p>(
    msg: impl Into<String>,
    profiles: &'p [&'p Profile],
    filter: Option<String>,
) -> Result<Option<&'p Path>> {
    let filter = filter.unwrap_or_default();
    let found: Vec<_> = profiles
        .iter()
        .filter(|p| cmp_profile(p, &filter))
        .map(Deref::deref)
        .collect();
    let selected = match found.len() {
        0 => {
            eprintln!("No profiles found that matched `{filter}`");
            profiles_prompt(msg, profiles)?
        }
        1 => found.first().map(|p| p.path()),
        _ => {
            if !filter.is_empty() {
                eprintln!("Found multiple profiles matching `{filter}`");
            }
            profiles_prompt(msg, &found)?
        }
    };

    Ok(selected)
}

pub fn check_profile_path(output_dir: &Path) -> Result<()> {
    if output_dir.is_relative() {
        bail!("The profile directory must be given as an absolute path");
    }
    Ok(())
}

/// Allow loose matching of profiles by either an exact name, or by path suffix
fn cmp_profile(profile: &Profile, name: &str) -> bool {
    profile.name() == name || profile.path().ends_with(name)
}

fn profiles_prompt<'p>(
    msg: impl Into<String>,
    profiles: &[&'p Profile],
) -> Result<Option<&'p Path>> {
    // let profiles = profiles.as_ref();
    let mut prompt = Select::with_theme(&*THEME).with_prompt(msg);
    for p in profiles {
        // Adding individually avoids allocating all the Strings twice
        prompt = prompt.item(fmt_profile_simple(p, 35, 35));
    }
    prompt
        .interact_opt()
        .map(|choice| choice.map(|i| profiles[i].path()))
        .map_err(Into::into)
}

/// This is a macro to get around overzealous mutable borrow rules on config
macro_rules! switch_profile {
    ($config:expr, $profiles:expr, $switch_to:expr) => {{
        let selected = crate::subcommands::profile::pick_profile(
            "Select a new profile to set as active",
            &$profiles,
            $switch_to,
        )?;
        if let Some(selected) = selected {
            $config.set_active(selected.to_owned())?;
        }
    }};
}
pub(crate) use switch_profile;
