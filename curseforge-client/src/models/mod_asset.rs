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
pub struct ModAsset {
    #[serde(rename = "id")]
    pub id: u64,
    #[serde(rename = "modId")]
    pub mod_id: u64,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: ::url::Url,
    #[serde(rename = "url")]
    pub url: ::url::Url,
}

impl ModAsset {
    pub fn new(id: u64, mod_id: u64, title: String, description: String, thumbnail_url: ::url::Url, url: ::url::Url) -> Self {
        Self {
            id,
            mod_id,
            title,
            description,
            thumbnail_url,
            url,
        }
    }
}

