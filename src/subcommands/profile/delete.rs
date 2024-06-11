use std::{ops::Deref, path::Path};

use anyhow::{bail, Result};
use dialoguer::Select;
use relibium::{config::Profile, Config};

use crate::tui::{fmt_profile_simple, THEME};

const DEL_MSG: &str = "Select a profile to delete";
const SWITCH_MSG: &str = "Select a new profile to set as active after deletion";

pub fn delete(
    config: &mut Config,
    name: Option<String>,
    switch_to: Option<String>,
) -> Result<Profile> {
    let profiles = config.get_profiles();
    let selected = if let Some(ref name) = name {
        let found: Vec<_> = profiles
            .iter()
            .filter(|profile| check_match(profile, name))
            .map(Deref::deref)
            .collect();
        match found.len() {
            0 => bail!("No profiles found that matched `{name}`"),
            1 => found.first().map(|p| p.path()),
            _ => {
                eprintln!("Found multiple profiles matching `{name}`");
                prompt(DEL_MSG, &found)?
            }
        }
    } else {
        prompt(DEL_MSG, &profiles)?
    };
    if selected.is_none() {
        // User cancelled
        bail!("Cancelled")
    }
    let selected = selected.unwrap().to_owned();

    // If the currently selected profile is being removed
    if profiles.len() > 2 && config.active().is_some_and(|a| a == &selected) {
        eprintln!("Switching active profile before deletion...");
        let switch_to = switch_to.unwrap_or_default();
        let found: Vec<_> = profiles
            .iter()
            .filter(|p| p.path() != selected)
            .filter(|p| check_match(p, &switch_to))
            .map(Deref::deref)
            .collect();
        let selected = match found.len() {
            0 => {
                eprintln!("No profiles found that matched `{switch_to}`");
                let profiles: Vec<_> = profiles
                    .iter()
                    .filter(|p| p.path() != selected)
                    .map(Deref::deref)
                    .collect();
                prompt(SWITCH_MSG, &profiles)?
            }
            1 => found.first().map(|p| p.path()),
            _ => {
                eprintln!("Found multiple profiles matching `{switch_to}`");
                prompt(SWITCH_MSG, &found)?
            }
        };
        if let Some(selected) = selected {
            config.set_active(selected.to_owned());
        }
    }

    config.remove_profile(selected).map_err(Into::into)
}

/// Allow loose matching of profiles by either an exact name, or by path suffix
fn check_match(profile: &Profile, name: &str) -> bool {
    profile.name() == name || profile.path().ends_with(name)
}

fn prompt<'p>(msg: impl Into<String>, profiles: &[&'p Profile]) -> Result<Option<&'p Path>> {
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
