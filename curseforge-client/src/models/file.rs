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
pub struct File {
    /// The file id
    #[serde(rename = "id")]
    pub id: u64,
    /// The game id related to the mod that this file belongs to
    #[serde(rename = "gameId")]
    pub game_id: u64,
    /// The mod id
    #[serde(rename = "modId")]
    pub mod_id: u64,
    /// Whether the file is available to download
    #[serde(rename = "isAvailable")]
    pub is_available: bool,
    /// Display name of the file
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// Exact file name
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "releaseType")]
    pub release_type: models::FileReleaseType,
    #[serde(rename = "fileStatus")]
    pub file_status: models::FileStatus,
    /// The file hash (i.e. md5 or sha1)
    #[serde(rename = "hashes")]
    pub hashes: Vec<models::FileHash>,
    /// The file timestamp
    #[serde(rename = "fileDate")]
    pub file_date: String,
    /// The file length in bytes
    #[serde(rename = "fileLength")]
    pub file_length: u64,
    /// The number of downloads for the file
    #[serde(rename = "downloadCount")]
    pub download_count: u64,
    #[serde(rename = "downloadUrl", skip_serializing_if = "Option::is_none")]
    pub download_url: Option<::url::Url>,
    /// List of game versions this file is relevant for
    #[serde(rename = "gameVersions")]
    pub game_versions: Vec<String>,
    /// Metadata used for sorting by game versions
    #[serde(rename = "sortableGameVersions")]
    pub sortable_game_versions: Vec<models::SortableGameVersion>,
    /// List of dependencies files
    #[serde(rename = "dependencies")]
    pub dependencies: Vec<models::FileDependency>,
    #[serde(rename = "exposeAsAlternative", skip_serializing_if = "Option::is_none")]
    pub expose_as_alternative: Option<bool>,
    #[serde(rename = "parentProjectFileId", skip_serializing_if = "Option::is_none")]
    pub parent_project_file_id: Option<u64>,
    #[serde(rename = "alternateFileId", skip_serializing_if = "Option::is_none")]
    pub alternate_file_id: Option<u64>,
    #[serde(rename = "isServerPack", skip_serializing_if = "Option::is_none")]
    pub is_server_pack: Option<bool>,
    #[serde(rename = "serverPackFileId", skip_serializing_if = "Option::is_none")]
    pub server_pack_file_id: Option<u64>,
    #[serde(rename = "isEarlyAccessContent", skip_serializing_if = "Option::is_none")]
    pub is_early_access_content: Option<bool>,
    #[serde(rename = "earlyAccessEndDate", skip_serializing_if = "Option::is_none")]
    pub early_access_end_date: Option<String>,
    #[serde(rename = "fileFingerprint")]
    pub file_fingerprint: u64,
    #[serde(rename = "modules")]
    pub modules: Vec<models::FileModule>,
}

impl File {
    pub fn new(id: u64, game_id: u64, mod_id: u64, is_available: bool, display_name: String, file_name: String, release_type: models::FileReleaseType, file_status: models::FileStatus, hashes: Vec<models::FileHash>, file_date: String, file_length: u64, download_count: u64, game_versions: Vec<String>, sortable_game_versions: Vec<models::SortableGameVersion>, dependencies: Vec<models::FileDependency>, file_fingerprint: u64, modules: Vec<models::FileModule>) -> Self {
        Self {
            id,
            game_id,
            mod_id,
            is_available,
            display_name,
            file_name,
            release_type,
            file_status,
            hashes,
            file_date,
            file_length,
            download_count,
            game_versions,
            sortable_game_versions,
            dependencies,
            file_fingerprint,
            modules,
            download_url: None,
            expose_as_alternative: None,
            parent_project_file_id: None,
            alternate_file_id: None,
            is_server_pack: None,
            server_pack_file_id: None,
            is_early_access_content: None,
            early_access_end_date: None,
        }
    }
}

