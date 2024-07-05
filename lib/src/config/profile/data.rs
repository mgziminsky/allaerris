use std::{
    collections::{BinaryHeap, HashSet},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    config::{Mod, ModLoader, Modpack},
    fs_util::{FsUtil, FsUtils},
    PathAbsolute, Result,
};


/// Macro for consts used in docs
macro_rules! consts {
    (DEFAULT_GAME_VERSION) => {
        "1.21"
    };
    (FILENAME) => {
        concat!(".", env!("CARGO_PKG_NAME"), "-profile")
    };
}
pub(crate) use consts;

/// Minecraft version used for [`ProfileData::default()`]
pub const DEFAULT_GAME_VERSION: &str = consts!(DEFAULT_GAME_VERSION);

/// Name of file used to [save]/[load] [profiles](ProfileData) from the
/// filesystem
///
/// [save]: ProfileData::save_to
/// [load]: ProfileData::load
pub const FILENAME: &str = consts!(FILENAME);


/// All the data needed to set up a modded instance of Minecraft
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProfileData {
    #[doc = concat!("The version of Minecraft to install mods for [Default: ", consts!(DEFAULT_GAME_VERSION) ,"]")]
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

macro_rules! remove_sorted {
    ($list:expr, $idxs:expr) => {
        $idxs.map(|idx| $list.swap_remove(*idx)).collect()
    };
}
impl ProfileData {
    #[doc = concat!("Attempt to load a [profile](Self) from a file named `", consts!(FILENAME), "` located at `path`")]
    ///
    /// # Errors
    /// Will return any errors that occur while trying to read or parse the file
    pub async fn load(path: impl AsRef<PathAbsolute>) -> Result<Self> {
        FsUtil::load_file(&path.as_ref().join(FILENAME)).await
    }

    #[doc = concat!("Attempt to save the [profile](Self) to a file named `", consts!(FILENAME), "` located at `path`")]
    ///
    /// # Errors
    /// Will return any errors that occur while trying to write the file
    pub async fn save_to(&self, path: impl AsRef<PathAbsolute>) -> Result<()> {
        FsUtil::save_file(self, &path.as_ref().join(FILENAME)).await
    }

    /// Returns true if both [`mods`](Self::mods) and
    /// [`modpack`](Self::modpack) are empty
    pub fn is_empty(&self) -> bool {
        self.mods.is_empty() && self.modpack.is_none()
    }

    /// Attempt to add all `mods` to this profile. Only adds if not already
    /// present based on [`id`](Mod::id). Returns a list of [`Result`] where
    /// [`Ok`] means the mod was added, and [`Err`] means it was already
    /// present.
    pub fn add_mods<'m>(&mut self, mods: impl IntoIterator<Item = &'m Mod>) -> Vec<std::result::Result<&'m Mod, &'m Mod>> {
        let set = HashSet::<_>::from_iter(&self.mods);

        let checked: Vec<_> = mods.into_iter().map(|m| if set.contains(m) { Err(m) } else { Ok(m) }).collect();

        let to_add = checked.iter().filter_map(|r| r.ok());
        self.mods.reserve(to_add.clone().count());
        self.mods.extend(to_add.cloned());

        checked
    }

    /// Checks each value in `to_remove` against the [name](Mod::name) and
    /// [id](Mod::id) of all [mods](Mod), removing any that match and returning
    /// them
    ///
    /// This does not preserve ordering of the remaining mods after removal
    pub fn remove_mods_matching<'a>(&mut self, to_remove: impl AsRef<[&'a str]>) -> Vec<Mod> {
        // Convert all to lowercase once up front
        let to_remove: Vec<_> = to_remove.as_ref().iter().map(|s| (*s, s.to_lowercase())).collect();

        let idxs = self.mods.iter().enumerate().fold(vec![], |mut found, (idx, m)| {
            if to_remove
                .iter()
                .any(|(rm_id, rm_name)| rm_name == &m.name.to_lowercase() || rm_id == &m.id().to_string())
            {
                found.push(idx);
            }
            found
        });

        remove_sorted!(self.mods, idxs.iter().rev())
    }

    /// Removes all mods in `indices` and returns them
    ///
    /// This does not preserve ordering of the remaining mods after removal
    /// # Panics
    /// Panics if any `index` is out of bounds.
    pub fn remove_mods_at(&mut self, indices: impl AsRef<[usize]>) -> Vec<Mod> {
        remove_sorted!(self.mods, BinaryHeap::from_iter(indices.as_ref()).into_iter())
    }

    /// Returns the path where this [`ProfileData`] would be saved given the
    /// provided base path
    pub fn file_path(path: impl AsRef<Path>) -> PathBuf {
        path.as_ref().join(FILENAME)
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
