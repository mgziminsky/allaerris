use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{Mod, ModLoader, Modpack};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    pub name: String,

    /// The directory to download mod files to
    pub output_dir: PathBuf,

    /// Only download mod files compatible with this Minecraft version
    pub game_version: String,

    #[serde(default, skip_serializing_if = "is_unknown")]
    pub loader: ModLoader,

    pub mods: Vec<Mod>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modpack: Option<Modpack>,
}

fn is_unknown(loader: &ModLoader) -> bool {
    matches!(loader, ModLoader::Unknown)
}
