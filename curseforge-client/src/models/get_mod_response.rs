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
pub struct GetModResponse {
    #[serde(rename = "data")]
    pub data: models::Mod,
}

impl GetModResponse {
    pub fn new(data: models::Mod) -> Self {
        Self {
            data,
        }
    }
}

