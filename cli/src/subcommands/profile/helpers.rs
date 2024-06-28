use std::ops::Deref;

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Input, Select};
use relibium::{
    checked_types::PathAbsolute,
    config::{profile::DEFAULT_GAME_VERSION, ModLoader, Profile},
};
use tokio::sync::OnceCell;

use crate::tui::{fmt_profile_simple, THEME};

static MC_VERSIONS: OnceCell<Vec<String>> = OnceCell::const_new();

pub fn pick_mod_loader(default: Option<ModLoader>) -> Result<ModLoader> {
    let mut picker = Select::with_theme(&*THEME).with_prompt("Select the mod loader to use:").items(&[
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

pub async fn pick_minecraft_version(default: Option<&str>) -> Result<String> {
    let versions = MC_VERSIONS.get_or_try_init(fetch_versions).await;
    let choice = match versions {
        Result::Ok(versions) => Select::with_theme(&*THEME)
            .with_prompt("Which version of Minecraft should this profile use?")
            .items(versions)
            .default(default.and_then(|d| versions.iter().position(|v| v == d)).unwrap_or(0))
            .interact()
            .map(|i| versions[i].clone())?,
        err => {
            let err = err.context("Failed to load minecraft versions".bold()).unwrap_err();
            eprintln!("{}", format!("{:#}", err).red());
            Input::with_theme(&*THEME)
                .with_prompt("Enter Minecraft version for the profile:")
                .with_initial_text(default.unwrap_or(DEFAULT_GAME_VERSION))
                .interact_text()?
        },
    };
    Ok(choice)
}

pub fn pick_profile<'p>(msg: impl Into<String>, profiles: &'p [&'p Profile], filter: Option<String>) -> Result<Option<&'p PathAbsolute>> {
    let filter = filter.unwrap_or_default();
    let found: Vec<_> = profiles.iter().filter(|p| cmp_profile(p, &filter)).map(Deref::deref).collect();
    let selected = match found.len() {
        0 => {
            eprintln!("No profiles found that matched `{filter}`");
            profiles_prompt(msg, profiles)?
        },
        1 => found.first().map(|p| &p.path),
        _ => {
            if !filter.is_empty() {
                eprintln!("Found multiple profiles matching `{filter}`");
            }
            profiles_prompt(msg, &found)?
        },
    };

    Ok(selected)
}

/// Allow loose matching of profiles by either an exact name, or by path suffix
fn cmp_profile(profile: &Profile, name: &str) -> bool {
    profile.name() == name || profile.path.ends_with(name)
}

fn profiles_prompt<'p>(msg: impl Into<String>, profiles: &[&'p Profile]) -> Result<Option<&'p PathAbsolute>> {
    let mut prompt = Select::with_theme(&*THEME).with_prompt(msg);
    for p in profiles {
        // Adding individually avoids allocating all the Strings twice
        prompt = prompt.item(fmt_profile_simple(p, 120));
    }
    prompt
        .interact_opt()
        .map(|choice| choice.map(|i| &profiles[i].path))
        .map_err(Into::into)
}

async fn fetch_versions() -> Result<Vec<String>> {
    use serde::Deserialize;
    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum ReleaseType {
        Release,
        #[serde(other)]
        Other,
    }
    #[derive(Deserialize)]
    struct Version {
        id: String,
        r#type: ReleaseType,
    }
    #[derive(Deserialize)]
    struct Manifest {
        versions: Vec<Version>,
    }
    reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await?
        .json::<Manifest>()
        .await
        .map(|r| {
            r.versions
                .into_iter()
                .filter_map(|v| match v.r#type {
                    ReleaseType::Release => Some(v.id),
                    ReleaseType::Other => None,
                })
                .collect()
        })
        .map_err(Into::into)
}

/// This is a macro to get around overzealous mutable borrow rules on config
macro_rules! switch_profile {
    ($config:expr, $profiles:expr, $switch_to:expr) => {{
        let selected = crate::subcommands::profile::helpers::pick_profile("Select a new profile to set as active", &$profiles, $switch_to)?;
        if let Some(selected) = selected {
            $config.set_active(selected.to_owned())?;
        }
    }};
}
pub(crate) use switch_profile;
