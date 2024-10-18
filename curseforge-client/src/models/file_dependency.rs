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
pub struct FileDependency {
    #[serde(rename = "modId")]
    pub mod_id: u64,
    #[serde(rename = "relationType")]
    pub relation_type: models::FileRelationType,
}

impl FileDependency {
    pub fn new(mod_id: u64, relation_type: models::FileRelationType) -> Self {
        Self {
            mod_id,
            relation_type,
        }
    }
}

