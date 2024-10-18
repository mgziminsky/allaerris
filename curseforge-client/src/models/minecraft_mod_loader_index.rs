/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.240719
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinecraftModLoaderIndex {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "gameVersion")]
    pub game_version: String,
    #[serde(rename = "latest")]
    pub latest: bool,
    #[serde(rename = "recommended")]
    pub recommended: bool,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "type")]
    pub r#type: models::ModLoaderType,
}

impl MinecraftModLoaderIndex {
    pub fn new(name: String, game_version: String, latest: bool, recommended: bool, date_modified: String, r#type: models::ModLoaderType) -> Self {
        Self {
            name,
            game_version,
            latest,
            recommended,
            date_modified,
            r#type,
        }
    }
}

