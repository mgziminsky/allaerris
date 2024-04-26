use anyhow::{bail, Context, Ok, Result};
use colored::Colorize;
use dialoguer::Confirm;
use libium::config::structs::{Config, Modpack, ModpackIdentifier};

use crate::THEME;

pub fn delete(config: &mut Config, force: bool) -> Result<()> {
    let profile = config.active_profile_mut().context("No active profile")?;
    if let Some(modpack) = profile.modpack.as_ref() {
        if force
            || Confirm::with_theme(&*THEME)
                .default(true)
                .with_prompt(format!(
                    "Remove modpack {} from active profile?",
                    format_modpack(modpack)
                ))
                .interact()?
        {
            profile.modpack = None;
        }
        Ok(())
    } else {
        bail!("No modpack on active profile")
    }
}

fn format_modpack(modpack: &Modpack) -> String {
    let (ty, id) = match &modpack.identifier {
        ModpackIdentifier::CurseForgeModpack(id) => ("CF".red(), id.to_string().dimmed()),
        ModpackIdentifier::ModrinthModpack(id) => ("MR".green(), id.dimmed()),
    };
    format!("{} {:8} {}", ty, id, modpack.name.bold())
}
