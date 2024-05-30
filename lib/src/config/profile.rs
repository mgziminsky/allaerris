use std::{ops::Deref, path::Path};

use serde::{Deserialize, Serialize};

use super::{fs_util::*, Mod, ModLoader, Modpack};
use crate::Result;

crate::sealed!();

macro_rules! consts {
    (MC) => {
        "1.20"
    };
    (ProfileFile) => {
        concat!(".", env!("CARGO_PKG_NAME"), "-profile")
    };
}

/// Minecraft version used for [ProfileData::default()]
pub const DEFAULT_GAME_VERSION: &str = consts!(MC);

/// Name of file used to [save]/[load] [profiles](ProfileData) from the
/// filesystem
///
/// [save]: ProfileData::save_to
/// [load]: ProfileData::load
pub const FILENAME: &str = consts!(ProfileFile);

fn cmp_name(a: &Mod, b: &Mod) -> std::cmp::Ordering {
    a.name.cmp(&b.name)
}


/// All the data needed to set up a modded instance of Minecraft
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProfileData {
    /// The version of Minecraft to install mods for
    pub game_version: String,

    /// The type of mod loader to install mods for
    #[serde(default, skip_serializing_if = "ModLoader::is_unknown")]
    pub loader: ModLoader,

    /// The list of mods directly managed by this profile.
    ///
    /// Any mods in this list will take priority over the same mod
    /// from the modpack if present. This can be used to override
    /// the version of some mods in a modpack while leaving the others
    /// unaffected.
    pub mods: Vec<Mod>,

    /// The modpack to use as the base for this profile
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modpack: Option<Modpack>,
}

impl ProfileData {
    #[doc = concat!("Attempt to load a [profile](Self) from a file named `", consts!(ProfileFile), "` located at `path`")]
    ///
    /// # Errors
    /// Will return any errors that occur while trying to read or parse the file
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        FsUtil::load_file(&path.as_ref().join(FILENAME)).await
    }

    #[doc = concat!("Attempt to save the [profile](Self) to a file named `", consts!(ProfileFile), "` located at `path`")]
    ///
    /// # Errors
    /// Will return any errors that occur while trying to write the file
    pub async fn save_to(&self, path: impl AsRef<Path>) -> Result<()> {
        FsUtil::save_file(self, &path.as_ref().join(FILENAME)).await
    }
}

impl Default for ProfileData {
    fn default() -> Self {
        Self {
            game_version: DEFAULT_GAME_VERSION.to_owned(),
            loader: Default::default(),
            mods: Default::default(),
            modpack: None,
        }
    }
}


/// Wrapper around [profile data](ProfileData) with the inclusion of a
/// [name](Self::name) and [path](Self::path).
pub struct ProfileBase<'c, P: DataType> {
    /// Profile Name
    pub name: &'c str,
    /// Path profile will be [saved](ProfileMut::save) to
    pub path: &'c Path,
    pub(super) data: P,
    pub(super) dirty: bool,
}
/// Immutable [ProfileBase] type
pub type Profile<'c, 'p> = ProfileBase<'c, &'p ProfileData>;
/// Mutable [ProfileBase] type
pub type ProfileMut<'c, 'p> = ProfileBase<'c, &'p mut ProfileData>;

/// Mutable function proxies for changing values in [`data`](ProfileData)
#[allow(missing_docs)]
impl ProfileMut<'_, '_> {
    pub fn set_game_version(&mut self, version: impl AsRef<str>) {
        let version = version.as_ref();
        if self.game_version != version {
            self.dirty = true;
            self.data.game_version = version.to_owned();
        }
    }

    pub fn set_loader(&mut self, loader: ModLoader) {
        if self.loader != loader {
            self.dirty = true;
            self.data.loader = loader;
        }
    }

    pub fn set_modpack(&mut self, modpack: Modpack) {
        if !self.modpack.as_ref().is_some_and(|mp| *mp == modpack) {
            self.dirty = true;
            self.data.modpack.replace(modpack);
        }
    }

    pub fn remove_modpack(&mut self) -> Option<Modpack> {
        let old = self.data.modpack.take();
        if old.is_some() {
            self.dirty = true;
        }
        old
    }

    /// Save the [profile data](ProfileData) to the filesystem at
    /// [`path`](Self::path), but only if any of the values have changed since
    /// the last save.
    ///
    /// Before saving, the mods list will be sorted by name. Initial values are
    /// not tracked, so it is possible the save will run with unchanged values
    /// if any were changed and then changed back.
    ///
    /// # Errors
    ///
    /// Will return any IO errors encountered while attempting to save to the
    /// filesystem
    pub async fn save(&mut self) -> Result<()> {
        if self.dirty {
            self.data.mods.sort_unstable_by(cmp_name);
            self.data.save_to(self.path).await?;
            self.dirty = false;
        }
        Ok(())
    }
}
impl Extend<Mod> for ProfileMut<'_, '_> {
    #[inline]
    fn extend<I: IntoIterator<Item = Mod>>(&mut self, iter: I) {
        let len = self.mods.len();
        self.data.mods.extend(iter);
        if len != self.mods.len() {
            self.dirty = true;
        }
    }
}


#[doc(hidden)]
pub trait DataType: Deref<Target = ProfileData> + Sealed {}
impl<T> DataType for T where T: Deref<Target = ProfileData> + Sealed {}
impl Sealed for &ProfileData {}
impl Sealed for &mut ProfileData {}
impl<T: DataType> Deref for ProfileBase<'_, T> {
    type Target = ProfileData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client::schema::ProjectId, Error};

    #[test]
    fn dirty_save() {
        let mut p = ProfileMut {
            name: "Test Profile",
            path: "/dev/null".as_ref(),
            data: &mut Default::default(),
            dirty: true,
        };
        assert!(
            matches!(crate::block_on(p.save()), Err(Error::TestStub)),
            "Save operation should have been attempted"
        );
        assert!(p.dirty, "`dirty` should still be true after failed save");

        p.path = "/pass/null".as_ref();
        assert!(crate::block_on(p.save()).is_ok(), "Save operation should have succeeded");
        assert!(!p.dirty, "`dirty` should be false after success");
    }

    #[test]
    fn not_dirty_save() {
        let mut p = ProfileMut {
            name: "Test Profile",
            path: "/dev/null".as_ref(),
            data: &mut Default::default(),
            dirty: false,
        };
        assert!(crate::block_on(p.save()).is_ok(), "Save operation should have been skipped");
    }

    #[test]
    fn save_sort() {
        let unsorted = [
            Mod {
                id: ProjectId::Forge(3),
                slug: "test-3".to_owned(),
                name: "Test 3".to_owned(),
                path: None,
            },
            Mod {
                id: ProjectId::Forge(1),
                slug: "test-1".to_owned(),
                name: "Test 1".to_owned(),
                path: None,
            },
            Mod {
                id: ProjectId::Forge(2),
                slug: "test-2".to_owned(),
                name: "Test 2".to_owned(),
                path: None,
            },
            Mod {
                id: ProjectId::Forge(0),
                slug: "test-0".to_owned(),
                name: "Test 0".to_owned(),
                path: None,
            },
        ];
        let sorted = {
            let mut s = unsorted.clone();
            s.sort_unstable_by(cmp_name);
            s
        };
        let mut p = ProfileMut {
            name: "Test Profile",
            path: "/pass/null".as_ref(),
            data: &mut Default::default(),
            dirty: true,
        };
        p.extend(unsorted);
        assert!(crate::block_on(p.save()).is_ok(), "Save operation should have succeeded");
        assert_eq!(sorted.as_slice(), p.mods, "Mods should be sorted after save");
    }
}
