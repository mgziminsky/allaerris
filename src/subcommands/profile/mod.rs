mod configure;
mod create;
mod delete;
mod switch;
pub use configure::configure;
pub use create::create;
pub use delete::delete;
pub use switch::switch;

use anyhow::{bail, Result};
use dialoguer::Select;
use relibium::config::ModLoader;
use std::path::Path;

use crate::tui::THEME;

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
    let versions = Ferinth::default().list_game_versions().await?;
    let mut major_versions = ["Show all", "Show release"] // Prepend additional options
        .into_iter()
        .chain(
            versions
                .iter()
                .filter(|v| v.major) // Only get major versions
                .map(|v| v.version.as_ref())
                .collect::<Vec<_>>(),
        )
        .collect::<Vec<_>>();
    let selected_version = Select::with_theme(&*THEME)
        .with_prompt("Which version of Minecraft do you play?")
        .items(&major_versions)
        .default(2)
        .interact()?;
    match selected_version {
        0 | 1 => {
            let mut versions = versions
                .into_iter()
                .filter(|v| selected_version == 0 || v.version_type == GameVersionType::Release)
                .map(|v| v.version)
                .collect::<Vec<_>>();
            let selected_version = Select::with_theme(&*THEME)
                .with_prompt("Which version of Minecraft do you play?")
                .items(&versions)
                .interact()?;
            Ok(versions.swap_remove(selected_version))
        }
        _ => Ok(major_versions.swap_remove(selected_version).to_owned()),
    }
}

pub fn check_profile_path(output_dir: &Path) -> Result<()> {
    if output_dir.is_relative() {
        bail!("The profile directory must be given as an absolute path");
    }
    Ok(())
}
