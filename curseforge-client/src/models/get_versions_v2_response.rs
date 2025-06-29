/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.250410
 * 
 * Generated by: https://openapi-generator.tech
 */

#[allow(unused_imports)]
use crate::models;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetVersionsV2Response {
    /// The response data
    #[serde(rename = "data", default)]
    pub data: Vec<models::GameVersionsByTypeV2>,
}

impl GetVersionsV2Response {
    #[allow(clippy::too_many_arguments)]
    pub fn new(data: Vec<models::GameVersionsByTypeV2>) -> Self {
        Self {
            data,
        }
    }
}

