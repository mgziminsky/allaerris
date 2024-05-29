use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{load_file, save_file, Mod, ModLoader, Modpack};
use crate::Result;

pub const DEFAULT_GAME_VERSION: &str = "1.20";
pub const FILENAME: &str = concat!(".", env!("CARGO_PKG_NAME"), "-profile");


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modpack: Option<Modpack>,

    #[serde(skip)]
    pub dirty: bool,
}

impl Profile {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        load_file(&path.as_ref().join(FILENAME)).await
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        save_file(self, &path.as_ref().join(FILENAME)).await
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            game_version: DEFAULT_GAME_VERSION.to_owned(),
            loader: Default::default(),
            mods: Default::default(),
            modpack: None,
            dirty: true,
        }
    }
}
