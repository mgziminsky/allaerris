use anyhow::{anyhow, bail, Context, Result};
use relibium::config::{Config, Profile};
use yansi::Paint;

macro_rules! consts {
    (APP_NAME) => {
        env!("CARGO_PKG_NAME")
    };
}
pub(crate) use consts;

use crate::tui::const_style;

pub const APP_NAME: &str = consts!(APP_NAME);


/// Get the active profile with error handling
pub(crate) fn get_active_profile(config: &mut Config) -> Result<&mut Profile> {
    config
        .active_profile_mut()
        .map_err(|err| match err.kind() {
            relibium::ErrorKind::NoProfiles => anyhow!(
                "There are no profiles configured, add a profile using `{}`",
                format!("{APP_NAME} profile create").bold().italic()
            ),
            _ => err.into(),
        })
        .context(const_style!("Failed to load active profile"; bold()))
}

/// Check if `profile` is empty, and if so return an error
pub(crate) async fn check_empty_profile(profile: &Profile) -> Result<()> {
    if profile.data().await?.is_empty() {
        bail!(
            "The currently selected profile is empty! Run `{}` to see how to add mods",
            const_style!(concat!(consts!(APP_NAME), " help"); bold())
        );
    }
    Ok(())
}
