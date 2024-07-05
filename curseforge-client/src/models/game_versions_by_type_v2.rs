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
pub struct GameVersionsByTypeV2 {
    #[serde(rename = "type")]
    pub r#type: u32,
    #[serde(rename = "versions", default)]
    pub versions: Vec<models::GameVersion>,
}

impl GameVersionsByTypeV2 {
    pub fn new(r#type: u32, versions: Vec<models::GameVersion>) -> Self {
        Self {
            r#type,
            versions,
        }
    }
}

