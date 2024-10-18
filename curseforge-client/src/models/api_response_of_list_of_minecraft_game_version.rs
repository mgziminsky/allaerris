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
pub struct ApiResponseOfListOfMinecraftGameVersion {
    #[serde(rename = "data", default)]
    pub data: Vec<models::MinecraftGameVersion>,
}

impl ApiResponseOfListOfMinecraftGameVersion {
    pub fn new(data: Vec<models::MinecraftGameVersion>) -> Self {
        Self {
            data,
        }
    }
}

