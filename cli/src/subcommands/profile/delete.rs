use std::ops::Deref;

use anyhow::{bail, Result};
use relibium::{config::Profile, Config};

use super::helpers::pick_profile;
use crate::subcommands::profile::switch_profile;

pub fn delete(config: &mut Config, name: Option<String>, switch_to: Option<String>) -> Result<Profile> {
    let profiles = config.get_profiles();
    let selected = pick_profile("Select a profile to delete", &profiles, name)?;
    if selected.is_none() {
        // User cancelled
        bail!("Cancelled")
    }
    let selected = selected.unwrap().to_owned();

    // If the currently selected profile is being removed
    if profiles.len() > 2 && config.active().is_some_and(|a| a == &selected) {
        eprintln!("Switching active profile before deletion...");
        let others: Vec<_> = profiles.iter().filter(|p| p.path() != &selected).map(Deref::deref).collect();
        switch_profile!(config, others, switch_to);
    }

    config.remove_profile(selected).map_err(Into::into)
}
