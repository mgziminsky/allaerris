#![allow(dead_code)]

use serde::Deserialize;

use crate::checked_types::PathScoped;


#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ModpackManifest {
    #[serde(rename_all = "camelCase")]
    V1 {
        manifest_version: super::version::Version<1>,
        manifest_type: ManifestType,
        minecraft: GameData,
        version: String,
        name: String,
        author: String,
        files: Vec<ManifestFile>,
        overrides: PathScoped,
    },
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ManifestType {
    MinecraftModpack,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameData {
    pub version: String,
    pub mod_loaders: Vec<ModLoader>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFile {
    pub project_id: u64,
    pub file_id: u64,
    pub required: bool,
}
