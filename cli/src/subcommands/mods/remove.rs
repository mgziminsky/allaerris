use anyhow::Result;
use dialoguer::MultiSelect;
use relibium::config::{profile::ProfileData, Mod};

use crate::tui::{mod_single_line, THEME};

/// If `to_remove` is empty, display a list of projects in the profile to select
/// from and remove the selected ones. Otherwise, search the given strings with
/// the projects' name and IDs and remove them
pub fn remove(profile: &mut ProfileData, to_remove: &[String]) -> Result<Vec<Mod>> {
    Ok(if to_remove.is_empty() {
        let indices = prompt(&profile.mods)?;
        profile.remove_mods_at(indices)
    } else {
        let to_remove = to_remove.iter().map(String::as_str).collect::<Vec<_>>();
        profile.remove_mods_matching(to_remove)
    })
}

fn prompt(mods: &[Mod]) -> Result<Vec<usize>> {
    Ok(
        match MultiSelect::with_theme(&*THEME)
            .with_prompt("Select mods to remove")
            .items(&mods.iter().map(mod_single_line).collect::<Vec<_>>())
            .report(false)
            .interact_opt()?
        {
            Some(selected) => selected,
            None => return Ok(vec![]),
        },
    )
}
