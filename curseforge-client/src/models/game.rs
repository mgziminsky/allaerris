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
pub struct Game {
    #[serde(rename = "id")]
    pub id: u32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "slug")]
    pub slug: String,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "assets")]
    pub assets: models::GameAssets,
    #[serde(rename = "status")]
    pub status: models::CoreStatus,
    #[serde(rename = "apiStatus")]
    pub api_status: models::CoreApiStatus,
}

impl Game {
    pub fn new(id: u32, name: String, slug: String, date_modified: String, assets: models::GameAssets, status: models::CoreStatus, api_status: models::CoreApiStatus) -> Self {
        Self {
            id,
            name,
            slug,
            date_modified,
            assets,
            status,
            api_status,
        }
    }
}

