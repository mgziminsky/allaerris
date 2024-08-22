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
pub struct GetGameResponse {
    #[serde(rename = "data")]
    pub data: models::Game,
}

impl GetGameResponse {
    pub fn new(data: models::Game) -> Self {
        Self {
            data,
        }
    }
}

