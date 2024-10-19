/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.240719
 * 
 * Generated by: https://openapi-generator.tech
 */

#[allow(unused_imports)]
use crate::models;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetFeaturedModsRequestBody {
    #[serde(rename = "gameId")]
    pub game_id: u64,
    #[serde(rename = "excludedModIds", default)]
    pub excluded_mod_ids: Vec<u64>,
    #[serde(rename = "gameVersionTypeId", skip_serializing_if = "Option::is_none")]
    pub game_version_type_id: Option<u64>,
}

impl GetFeaturedModsRequestBody {
    #[allow(clippy::too_many_arguments)]
    pub fn new(game_id: u64, excluded_mod_ids: Vec<u64>) -> Self {
        Self {
            game_id,
            excluded_mod_ids,
            game_version_type_id: None,
        }
    }
}

