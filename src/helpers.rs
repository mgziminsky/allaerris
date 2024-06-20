use anyhow::{anyhow, bail, Context, Ok, Result};
use colored::Colorize;
use relibium::config::{Config, Profile};

macro_rules! consts {
    (APP_NAME) => {
        env!("CARGO_PKG_NAME")
    };
}
pub(crate) use consts;

pub const APP_NAME: &str = consts!(APP_NAME);

const MSG_PROFILE_EMPTY: &str = concat!(
    "The currently selected profile is empty! Run `",
    consts!(APP_NAME),
    " help` to see how to add mods"
);

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
        .with_context(|| "Failed to load active profile".bold())
}

/// Check if `profile` is empty, and if so return an error
pub(crate) async fn check_empty_profile(profile: &Profile) -> Result<()> {
    if profile.data().await?.is_empty() {
        bail!(MSG_PROFILE_EMPTY);
    }
    Ok(())
}
