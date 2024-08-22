/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinecraftGameVersion {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "gameVersionId")]
    pub game_version_id: u32,
    #[serde(rename = "versionString")]
    pub version_string: String,
    #[serde(rename = "jarDownloadUrl")]
    pub jar_download_url: ::url::Url,
    #[serde(rename = "jsonDownloadUrl")]
    pub json_download_url: ::url::Url,
    #[serde(rename = "approved")]
    pub approved: bool,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: u32,
    #[serde(rename = "gameVersionStatus")]
    pub game_version_status: models::GameVersionStatus,
    #[serde(rename = "gameVersionTypeStatus")]
    pub game_version_type_status: models::GameVersionTypeStatus,
}

impl MinecraftGameVersion {
    pub fn new(id: u32, game_version_id: u32, version_string: String, jar_download_url: ::url::Url, json_download_url: ::url::Url, approved: bool, date_modified: String, game_version_type_id: u32, game_version_status: models::GameVersionStatus, game_version_type_status: models::GameVersionTypeStatus) -> Self {
        Self {
            id,
            game_version_id,
            version_string,
            jar_download_url,
            json_download_url,
            approved,
            date_modified,
            game_version_type_id,
            game_version_status,
            game_version_type_status,
        }
    }
}

