//! Configuration types used for managing and interacting with mods/modpacks on
//! the system
mod loader;
mod modpack;
mod mods;
mod project_with_version;
mod serde;

// Use attribute with newlines so mod docs aren't merged on the same line
#[doc = "Types relating to [profile data](profile::ProfileData)\n\n"]
pub mod profile;

use std::{collections::BTreeSet, path::Path};

use ::serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

#[doc(inline)]
pub use self::profile::Profile;
use self::profile::ProfileByPath;
pub use self::{loader::*, modpack::*, mods::*, project_with_version::*};
use crate::{
    fs_util::{FsUtil, FsUtils},
    ErrorKind, PathAbsolute, Result, CONF_DIR,
};

/// Full path to the default config file
pub static DEFAULT_CONFIG_PATH: Lazy<PathAbsolute> = Lazy::new(|| CONF_DIR.join("config.json"));

type ProfilesList = BTreeSet<ProfileByPath>;


/// Global config object containing a list of profile names and their path
///
/// The actual [profile data] is stored externally at the path associated with
/// the profile.
///
/// [profile data]: profile::ProfileData
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default, from = "ConfigDe")]
pub struct Config {
    /// Will only be [None] when [profiles](Config::profiles) is empty
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<PathAbsolute>,

    #[serde(
        skip_serializing_if = "ProfilesList::is_empty",
        serialize_with = "self::serde::profiles::serialize"
    )]
    profiles: ProfilesList,
}


/// Workaround for no support of split borrowing of `self` behind method calls
macro_rules! get {
    ($self:ident.active) => {
        $self.active.as_ref().ok_or(ErrorKind::NoProfiles)?
    };
    ($self:ident.profile_mut($path:expr)) => {
        $self
            .profiles
            .get($path)
            .map(ProfileByPath::force_mut)
            .ok_or(ErrorKind::UnknownProfile.into())
    };
}

// Profile
impl Config {
    /// Returns the path of the active profile if set
    pub fn active(&self) -> Option<&PathAbsolute> {
        self.active.as_ref()
    }

    /// Returns `true` if an [active profile](Self::active_profile) is set
    ///
    /// Will only be `false` when [profiles](Config::get_profiles) is empty
    pub fn has_active(&self) -> bool {
        self.active.is_some()
    }

    /// Sets the [active profile] to `path` and returns the previous
    /// [profile data] if it was loaded and `path` changed.
    ///
    /// The current [active profile] should be [saved](Profile::save) before
    /// changing, otherwise any modifications will be lost
    ///
    /// [active profile]: Self::active_profile
    /// [profile data]: profile::ProfileData
    /// # Errors
    ///
    /// [`ErrorKind::UnknownProfile`]: if `path` is not present in list of known
    /// profiles
    pub fn set_active(&mut self, path: impl AsRef<PathAbsolute>) -> Result<()> {
        let path = path.as_ref();
        if !self.profiles.contains(path) {
            return Err(ErrorKind::UnknownProfile)?;
        }
        if self.active.as_ref().is_some_and(|ap| ap != path) {
            self.active.replace(path.to_owned());
        }
        Ok(())
    }

    /// Returns the currently active [profile]
    ///
    /// This is the [profile] that actions that don't take an explicit profile
    /// will be applied to
    ///
    /// # Errors
    ///
    /// [`ErrorKind::NoProfiles`]: if [profiles](Profile) is empty
    ///
    /// [profiles]: Self::profiles
    pub fn active_profile(&self) -> Result<&Profile> {
        self.profile(get!(self.active))
    }

    /// See [`active_profile`](Self::active_profile)
    pub fn active_profile_mut(&mut self) -> Result<&mut Profile> {
        get!(self.profile_mut(get!(self.active)))
    }

    /// Return the profile associated with the given `path`
    ///
    /// # Errors
    ///
    /// [`ErrorKind::UnknownProfile`] if `path` is not present in list of
    /// known [profiles]
    ///
    /// [profiles]: Self::get_profiles
    pub fn profile(&self, path: impl AsRef<Path>) -> Result<&Profile> {
        self.profiles
            .get(path.as_ref())
            .map(AsRef::as_ref)
            .ok_or(ErrorKind::UnknownProfile.into())
    }

    /// See [`profile`](Self::profile)
    pub fn profile_mut(&mut self, path: impl AsRef<Path>) -> Result<&mut Profile> {
        get!(self.profile_mut(path.as_ref()))
    }

    /// The list of [profiles](Profile) sorted by `path`
    pub fn get_profiles(&self) -> Vec<&Profile> {
        self.profiles.iter().map(AsRef::as_ref).collect()
    }

    /// See [`get_profiles`](Self::get_profiles)
    pub fn get_profiles_mut(&mut self) -> Vec<&mut Profile> {
        self.profiles.iter().map(ProfileByPath::force_mut).collect()
    }

    /// Add the [profile](Profile) to this config if not already present
    ///
    /// # Errors
    ///
    /// This function will return an error containing the passed in profile
    /// if a profile with the same path is already present in the config
    pub fn add_profile(&mut self, profile: Profile) -> std::result::Result<(), Profile> {
        if self.profiles.contains(&*profile.path) {
            Err(profile)
        } else {
            self.profiles.insert(profile.into());
            Ok(())
        }
    }

    /// Remove and return the [profile](Profile) for `path`
    ///
    /// If the removed profile was the currently [active profile], then the
    /// active profile will be switched to the first profile
    ///
    /// # Errors
    ///
    /// [`ErrorKind::UnknownProfile`]: if no profile exists for `path`
    ///
    /// [active profile]: Self::active_profile
    pub fn remove_profile(&mut self, path: impl AsRef<Path>) -> Result<Profile> {
        let removed: Profile = self.profiles.take(path.as_ref()).map(Into::into).ok_or(ErrorKind::UnknownProfile)?;
        if self.active.as_ref().is_some_and(|a| a == &removed.path) {
            self.active = self.profiles.first().map(|p| p.as_absolute().to_owned());
        }
        Ok(removed)
    }
}

// Load/Save
impl Config {
    /// Load a [config](Config) from the file located at the
    /// [default config path]
    ///
    /// [default config path]: DEFAULT_CONFIG_PATH
    /// # Errors
    ///
    /// Will return any IO or parse errors encountered while attempting to read
    /// the config
    pub async fn load() -> Result<Self> {
        Self::load_from(&*DEFAULT_CONFIG_PATH).await
    }

    /// Load a [config](Config) from the file located at `path`
    ///
    /// # Errors
    ///
    /// Will return any IO or parse errors encountered while attempting to read
    /// the config
    pub async fn load_from(path: impl AsRef<Path>) -> Result<Self> {
        FsUtil::load_file(path.as_ref()).await
    }

    /// Save this [config](Config) and the active [profile] to the file located
    /// at the [default config path]
    ///
    ///
    /// [profile]: profile::ProfileData::save_to
    /// [default config path]: DEFAULT_CONFIG_PATH
    /// # Errors
    ///
    /// Will return any IO errors encountered while attempting to save to the
    /// filesystem
    pub async fn save(&mut self) -> Result<()> {
        self.save_to(&*DEFAULT_CONFIG_PATH).await
    }

    /// Save this [config](Config) and the active [profile] to the file located
    /// at `path`
    ///
    /// [profile]: Profile::save
    /// # Errors
    ///
    /// Will return any IO errors encountered while attempting to save to the
    /// filesystem
    pub async fn save_to(&mut self, path: impl AsRef<Path>) -> Result<()> {
        FsUtil::save_file(self, path.as_ref()).await?;
        for profile in self.profiles.iter().map(ProfileByPath::force_mut) {
            profile.save().await?;
        }
        Ok(())
    }
}


/// Proxy deserialize object for [Config] to ensure data validity
#[derive(Deserialize, Default)]
#[serde(default)]
struct ConfigDe {
    active: Option<PathAbsolute>,
    #[serde(deserialize_with = "self::serde::profiles::deserialize")]
    profiles: ProfilesList,
}
impl From<ConfigDe> for Config {
    fn from(de: ConfigDe) -> Self {
        Self {
            active: de
                .active
                // Require active_profile to be present in profiles
                .and_then(|p| if de.profiles.contains(&p) { Some(p) } else { None })
                // Activate first profile from list if present and not already set
                .or_else(|| de.profiles.first().map(ProfileByPath::as_absolute).map(ToOwned::to_owned)),
            profiles: de.profiles,
        }
    }
}


#[cfg(test)]
mod tests {
    use std::iter::zip;

    use serde_test::{assert_de_tokens, assert_ser_tokens, Token};

    use super::*;

    static PATHS: Lazy<[PathAbsolute; 3]> = Lazy::new(|| {
        [
            PathAbsolute::new("/test/profile/path/1").unwrap(),
            PathAbsolute::new("/test/profile/path/2").unwrap(),
            PathAbsolute::new("/test/profile/path/3").unwrap(),
        ]
    });
    const NAMES: &[&str] = &["Profile 1", "Profile 2", "Profile 3"];

    impl PartialEq for Config {
        fn eq(&self, other: &Self) -> bool {
            self.active == other.active && self.profiles == other.profiles
        }
    }

    fn _test_config() -> Config {
        Config {
            active: Some(PATHS[2].clone()),
            profiles: zip(NAMES, &*PATHS)
                .map(|(name, path)| Profile::new((*name).to_string(), path.clone()))
                .map(Into::into)
                .collect(),
        }
    }
    fn _test_ser_data() -> (Config, Vec<Token>) {
        let config = _test_config();
        let mut tokens = vec![
            Token::Struct { name: "Config", len: 2 },
            Token::Str("active"),
            Token::Some,
            Token::NewtypeStruct { name: "PathAbsolute" },
            Token::Str(PATHS[2].to_str().unwrap()),
            Token::Str("profiles"),
            Token::Map { len: Some(PATHS.len()) },
        ];
        tokens.extend(zip(&*PATHS, NAMES).flat_map(|(p, n)| {
            [
                Token::NewtypeStruct { name: "PathAbsolute" },
                Token::Str(p.to_str().unwrap()),
                Token::Str(n),
            ]
        }));
        tokens.extend([Token::MapEnd, Token::StructEnd]);

        (config, tokens)
    }
    fn _test_de_data() -> (Config, Vec<Token>) {
        let (config, mut tokens) = _test_ser_data();
        if let Token::Struct { name, .. } = tokens.first_mut().unwrap() {
            *name = "ConfigDe";
        }
        tokens.retain_mut(|t| !matches!(t, Token::NewtypeStruct { .. }));
        (config, tokens)
    }

    #[test]
    fn serialize() {
        let (config, tokens) = _test_ser_data();
        eprintln!("{}", serde_json::to_string_pretty(&config).unwrap());
        assert_ser_tokens(&config, &tokens);
    }

    #[test]
    fn deserialize_all() {
        let (config, tokens) = _test_de_data();
        assert_de_tokens(&config, &tokens);
    }

    /// When no `active_profile` is set, then it should get set to the first
    /// listed profile automatically
    #[test]
    fn deserialize_no_active() {
        let (mut config, mut tokens) = _test_de_data();
        config.active.replace(PATHS[0].clone()); // Set active_profile to first path
        tokens.drain(1..=3); // Remove active_profile from tokens
        assert_de_tokens(&config, &tokens);
    }

    /// When the `active_profile` is set to a value not in the list of profiles,
    /// then it should get set to the first listed profile automatically
    #[test]
    fn deserialize_bad_active() {
        let (mut config, mut tokens) = _test_de_data();
        config.active.replace(PATHS[0].clone()); // Set active_profile to first path
        tokens[3] = Token::Str("/some/invalid/path"); // Set active_profile token to path not in profiles
        assert_de_tokens(&config, &tokens);
    }

    #[test]
    fn set_active_invalid() {
        let mut c = _test_config();
        let res = c.set_active(PathAbsolute::new("/some/invalid/path").unwrap());
        assert!(
            matches!(res, Err(ref e) if matches!(e.kind(), ErrorKind::UnknownProfile)),
            "set_active should fail with correct error given a path not in profiles: {res:?}"
        );
    }
}
