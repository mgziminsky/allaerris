use std::{borrow::Cow, ffi::OsStr, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use ferrallay::config::{Config, Profile, profile::ProfileData};
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
    // Check if we are inside a profile dir, and if so, use that profile
    if let Ok(cd) = std::path::absolute(".") {
        if let Some(path) = config.get_profiles().into_iter().map(Profile::path).find(|p| cd.starts_with(p)) {
            // WORKAROUND: https://rust-lang.github.io/rfcs/2094-nll.html
            // SAFETY: Conversion is done directly from as_encoded_bytes() values
            let path = unsafe {
                OsStr::from_encoded_bytes_unchecked(&cd.as_os_str().as_encoded_bytes()[..path.as_os_str().as_encoded_bytes().len()])
            };
            return Ok(config.profile_mut(path).expect("Profile should exist"));
        }
    }
    config
        .active_profile_mut()
        .map_err(|err| match err.kind() {
            ferrallay::ErrorKind::NoProfiles => anyhow!(
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

/// Look for a profile file in `path`, or the current directory if [`None`], and
/// its ancestors
pub fn path_profile(path: Option<&Path>) -> Option<Profile> {
    path.map(Cow::from)
        .or_else(|| std::path::absolute(".").map(Into::into).ok())
        .as_deref()
        .map(Path::ancestors)
        .and_then(|mut anc| anc.find(|p| ProfileData::file_path(p).exists()))
        .map(|p| Profile::new("Local Directory".to_owned(), p.try_into().expect("Should be an absolute path")))
}
