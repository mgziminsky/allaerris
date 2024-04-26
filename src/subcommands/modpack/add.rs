use super::check_output_directory;
use crate::{file_picker::pick_folder, THEME, TICK};
use anyhow::{anyhow, Result};
use colored::Colorize;
use dialoguer::Confirm;
use ferinth::{structures::project::Project, Ferinth};
use furse::{structures::mod_structs::Mod, Furse};
use libium::{
    config::structs::{Config, ModLoader, Modpack, ModpackIdentifier, Profile},
    get_minecraft_dir,
    modpack::add,
};
use std::{path::PathBuf, str::FromStr};

enum ProfileSource {
    Forge(Mod),
    Modrinth(Project),
}

impl ProfileSource {
    fn name(&self) -> &str {
        match self {
            Self::Forge(p) => &p.name,
            Self::Modrinth(p) => &p.title,
        }
    }

    fn into_profile(self, output_dir: PathBuf, install_overrides: bool) -> Profile {
        match self {
            Self::Forge(project) => Profile {
                name: project.name.clone(),
                output_dir: output_dir.join("mods"),
                game_version: String::from("modpack"),
                mod_loader: ModLoader::Unknown,
                mods: vec![],
                modpack: Some(Modpack {
                    name: project.name,
                    identifier: ModpackIdentifier::CurseForgeModpack(project.id),
                    output_dir,
                    install_overrides,
                }),
            },
            Self::Modrinth(project) => Profile {
                name: project.title.clone(),
                output_dir: output_dir.join("mods"),
                game_version: project
                    .game_versions
                    .iter()
                    .max()
                    .map(Clone::clone)
                    .unwrap_or_default(),
                mod_loader: project
                    .loaders
                    .iter()
                    .filter_map(|l| ModLoader::from_str(l).ok())
                    .next()
                    .unwrap_or(ModLoader::Unknown),
                mods: vec![],
                modpack: Some(Modpack {
                    name: project.title,
                    identifier: ModpackIdentifier::ModrinthModpack(project.id),
                    output_dir,
                    install_overrides,
                }),
            },
        }
    }
}

impl From<Mod> for ProfileSource {
    fn from(value: Mod) -> Self {
        Self::Forge(value)
    }
}

impl From<Project> for ProfileSource {
    fn from(value: Project) -> Self {
        Self::Modrinth(value)
    }
}

pub async fn add(
    identifier: String,
    config: &mut Config,
    output_dir: Option<PathBuf>,
    install_overrides: Option<bool>,
    forge: &Furse,
    modrinth: &Ferinth,
) -> Result<(), anyhow::Error> {
    let project: ProfileSource = if let Ok(project_id) = identifier.parse() {
        add::curseforge(forge, project_id).await?.into()
    } else {
        add::modrinth(modrinth, &identifier).await?.into()
    };
    println!("{} ({})", *TICK, project.name());
    println!("Where should the modpack be installed to?");
    let output_dir = match output_dir {
        Some(some) => some,
        None => pick_folder(
            &get_minecraft_dir(),
            "Pick an output directory",
            "Output Directory",
        )?
        .ok_or_else(|| anyhow!("Please pick an output directory"))?,
    };
    check_output_directory(&output_dir)?;
    let install_overrides = match install_overrides {
        Some(some) => some,
        None => Confirm::with_theme(&*THEME)
            .default(true)
            .with_prompt("Should overrides be installed?")
            .interact()?,
    };
    if install_overrides {
        println!(
            "{}",
            "WARNING: Files in your output directory may be overwritten by modpack overrides"
                .yellow()
                .bold()
        );
    }

    config
        .profiles
        .push(project.into_profile(output_dir, install_overrides));
    config.active_profile = config.profiles.len() - 1;

    Ok(())
}
