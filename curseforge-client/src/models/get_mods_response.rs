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
pub struct GetModsResponse {
    #[serde(rename = "data", default)]
    pub data: Vec<models::Mod>,
}

impl GetModsResponse {
    pub fn new(data: Vec<models::Mod>) -> Self {
        Self {
            data,
        }
    }
}

