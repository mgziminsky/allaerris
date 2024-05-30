//! Configuration types used for managing and interacting with mods/modpacks on
//! the system
mod fs_util;
mod loader;
mod modpack;
mod mods;

/// Types relating to profile [data](ProfileData) and [operations](ProfileBase)
pub mod profile;

use std::{
    collections::BTreeMap,
    io::ErrorKind,
    ops::Deref,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::{self, sync::OnceCell};

#[doc(inline)]
pub use self::profile::{Profile, ProfileMut};
use self::{
    fs_util::{FsUtil, FsUtils},
    profile::ProfileData,
};
pub use self::{loader::*, modpack::*, mods::*};
use crate::{Error, Result, CONF_DIR};

/// Full path to the default config file
pub static DEFAULT_CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| CONF_DIR.join("config.json"));


/// Global config object containing a list of profile names and their path
///
/// The actual [profile data] is stored externally at the path associated with
/// profile
///
/// [profile data]: ProfileBase
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(default, from = "ConfigDe")]
pub struct Config {
    /// Should only be [None] when [profiles](Config::profiles) is empty
    #[serde(skip_serializing_if = "Option::is_none")]
    active_profile: Option<PathBuf>,

    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    profiles: BTreeMap<PathBuf, String>,

    /// Active profile read from the filesystem will be stored here to prevent
    /// multiple round trips if accessed more than once
    #[serde(skip)]
    cache: OnceCell<ProfileData>,
}


/// Workaround for no support of split borrowing of `self` behind method calls
macro_rules! get_active {
    ($self:ident) => {
        if let Some(ref path) = $self.active_profile {
            $self.profiles.get_key_value(path).ok_or(Error::UnknownProfile)
        } else {
            Err(Error::NoProfiles)
        }
    };
}

// Profile
impl Config {
    /// Returns `true` if an [active profile](Self::active_profile) is set
    ///
    /// Will only be `false` when [profiles](Config::profiles) is empty
    pub fn has_active(&self) -> bool {
        self.active_profile.is_some()
    }

    /// # Errors
    ///
    /// [`Error::UnknownProfile`] if `path` is not present in list of known
    /// profiles
    pub fn set_active(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if !self.profiles.contains_key(path) {
            return Err(Error::UnknownProfile);
        }
        if self.active_profile.as_ref().is_some_and(|ap| ap != path) {
            self.cache.take();
            self.active_profile.replace(path.to_owned());
        }
        Ok(())
    }

    /// Workaround for no support of split borrowing of `self` behind method
    /// calls
    ///
    /// # Errors
    ///
    /// When loading the profile from the fs fails, the resulting error is
    /// returned
    async fn get_cached(&self, path: &Path) -> Result<&ProfileData> {
        self.cache
            .get_or_try_init(|| async {
                let p = match ProfileData::load(path).await {
                    Ok(p) => p,
                    Err(Error::IO(e)) if e.kind() == ErrorKind::NotFound => ProfileData::default(),
                    err => return err,
                };
                Ok(p)
            })
            .await
    }

    /// Returns the currently active [profile]
    ///
    /// This is the [profile] that actions that don't take an explicit profile
    /// will be applied to
    ///
    /// # Errors
    ///
    /// [[`Error::NoProfiles`]]: if [profiles] is empty
    ///
    /// [[`Error::UnknownProfile`]] if `path` is not present in list of known
    /// [profiles]
    ///
    /// [profile]: profile::ProfileBase
    /// [profiles]: Self::profiles
    pub async fn active_profile(&self) -> Result<Profile> {
        let (path, name) = get_active!(self)?;
        self.get_cached(path).await.map(|p| Profile {
            name,
            path,
            data: p,
            dirty: false,
        })
    }

    /// See [`active_profile()`](Config::active_profile())
    pub async fn active_profile_mut(&mut self) -> Result<ProfileMut> {
        // NOTE: This would ideally just call active_profile() and discard the immutable
        // profile for the error checking and init logic. Unfortunately, split borrows
        // don't work behind method calls, resulting in the whole self being immutably
        // borrowed
        let (path, name) = get_active!(self)?;
        self.get_cached(path).await?; // Only used for init code, discard result on success
        Ok(ProfileMut {
            name,
            path,
            data: self.cache.get_mut().expect("cached profile should have been initialized"),
            dirty: false,
        })
    }

    /// Mapping of [`Paths`](PathBuf) => profile name. Actual [profile data] is
    /// stored in a [config file] located at the `path`
    ///
    /// [profile data]: ProfileData
    /// [config file]: profile::FILENAME
    pub fn profiles(&self) -> &BTreeMap<PathBuf, String> {
        &self.profiles
    }
}

// Load/Save
impl Config {
    /// Load a [config](Config) from the file located at [default config path]
    ///
    /// [default config path]: DEFAULT_CONFIG_PATH
    /// # Errors
    ///
    /// Will return any IO or parse errors encountered while attempting to read
    /// the config
    pub async fn load() -> Result<Self> {
        Self::load_from(DEFAULT_CONFIG_PATH.deref()).await
    }

    /// Load a [config](Config) from the file located at `path`
    ///
    /// # Errors
    ///
    /// Will return any IO or parse errors encountered while attempting to read
    /// the config
    pub async fn load_from(path: impl AsRef<Path>) -> Result<Self> {
        let mut c: Config = FsUtil::load_file(path.as_ref()).await?;
        if c.active_profile.is_none() && !c.profiles.is_empty() {
            c.active_profile = c.profiles.first_key_value().map(|(p, _)| p.to_owned());
        }
        Ok(c)
    }

    /// Save this [config](Config) and the active [profile] to the file located
    /// at the [default config path]
    ///
    ///
    /// [profile]: profile::ProfileBase::save
    /// [default config path]: DEFAULT_CONFIG_PATH
    /// # Errors
    ///
    /// Will return any IO errors encountered while attempting to save to the
    /// filesystem
    pub async fn save(&self) -> Result<()> {
        self.save_to(DEFAULT_CONFIG_PATH.deref()).await
    }

    /// Save this [config](Config) and the active [profile] to the file located
    /// at the `path`
    ///
    ///
    /// [profile]: profile::ProfileBase::save
    /// # Errors
    ///
    /// Will return any IO errors encountered while attempting to save to the
    /// filesystem
    pub async fn save_to(&self, path: impl AsRef<Path>) -> Result<()> {
        FsUtil::save_file(self, path.as_ref()).await
    }
}


/// Proxy deserialize object for [Config] to ensure data validity
#[derive(Deserialize, Default)]
#[serde(default)]
struct ConfigDe {
    active_profile: Option<PathBuf>,
    profiles: BTreeMap<PathBuf, String>,
}
impl From<ConfigDe> for Config {
    fn from(de: ConfigDe) -> Self {
        Self {
            active_profile: de
                .active_profile
                // Require active_profile to be present in profiles
                .and_then(|p| if de.profiles.contains_key(&p) { Some(p) } else { None })
                // Activate first profile from list if present and not already set
                .or_else(|| de.profiles.first_key_value().map(|(p, _)| p.to_owned())),
            profiles: de.profiles,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use serde_test::{assert_de_tokens, assert_ser_tokens, Token};

    use super::*;

    const PATHS: &[&str] = &["/test/profile/path/1", "/test/profile/path/2", "/test/profile/path/3"];
    const NAMES: &[&str] = &["Profile 1", "Profile 2", "Profile 3"];

    impl PartialEq for Config {
        fn eq(&self, other: &Self) -> bool {
            self.active_profile == other.active_profile && self.profiles == other.profiles
        }
    }

    fn _test_config() -> Config {
        Config {
            active_profile: Some(PATHS[2].into()),
            profiles: BTreeMap::from_iter(zip(
                PATHS.iter().cloned().map(Into::into),
                NAMES.iter().cloned().map(ToOwned::to_owned),
            )),
            cache: OnceCell::new_with(Some(ProfileData::default())),
        }
    }
    fn _test_ser_data() -> (Config, Vec<Token>) {
        let config = _test_config();
        let mut tokens = vec![
            Token::Struct { name: "Config", len: 2 },
            Token::Str("active_profile"),
            Token::Some,
            Token::Str(PATHS[2]),
            Token::Str("profiles"),
            Token::Map { len: Some(PATHS.len()) },
        ];
        tokens.extend(zip(PATHS, NAMES).flat_map(|(p, n)| [*p, *n]).map(Token::Str));
        tokens.extend([Token::MapEnd, Token::StructEnd]);

        (config, tokens)
    }
    fn _test_de_data() -> (Config, Vec<Token>) {
        let (config, mut tokens) = _test_ser_data();
        if let Token::Struct { name, .. } = tokens.first_mut().unwrap() {
            *name = "ConfigDe";
        }
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

    /// When no active_profile is set, then it should get set to the first
    /// listed profile automatically
    #[test]
    fn deserialize_no_active() {
        let (mut config, mut tokens) = _test_de_data();
        config.active_profile.replace(PATHS[0].into()); // Set active_profile to first path
        tokens.drain(1..=3); // Remove active_profile from tokens
        assert_de_tokens(&config, &tokens);
    }

    /// When the active_profile is set to a value not in the list of profiles,
    /// then it should get set to the first listed profile automatically
    #[test]
    fn deserialize_bad_active() {
        let (mut config, mut tokens) = _test_de_data();
        config.active_profile.replace(PATHS[0].into()); // Set active_profile to first path
        tokens[3] = Token::Str("/some/invalid/path"); // Set active_profile token to path not in profiles
        assert_de_tokens(&config, &tokens);
    }

    #[test]
    fn set_active_invalid() {
        let mut c = _test_config();
        let res = c.set_active("/some/invalid/path");
        assert!(
            matches!(res, Err(Error::UnknownProfile)),
            "set_active should fail with correct error given a path not in profiles: {:?}",
            res
        );
        assert!(
            c.cache.initialized(),
            "cache should still be initialized after failed set: {:?}",
            c.cache
        );
    }

    #[test]
    fn set_active_same_keep_cache() {
        let mut c = _test_config();
        let path = c.active_profile.to_owned().unwrap();
        c.set_active(path).expect("set_active should succeed using same value");
        assert!(
            c.cache.initialized(),
            "cache should still be initialized after setting same value: {:?}",
            c.cache
        );
    }

    #[test]
    fn change_active_clear_cache() {
        let mut c = _test_config();
        c.set_active(PATHS[1]).expect("set_active should allow setting active_profile");
        assert!(
            !c.cache.initialized(),
            "cache should be cleared after changing active_profile: {:?}",
            c.cache
        );
    }
}
