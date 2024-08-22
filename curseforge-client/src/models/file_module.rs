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
pub struct FileModule {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "fingerprint")]
    pub fingerprint: usize,
}

impl FileModule {
    pub fn new(name: String, fingerprint: usize) -> Self {
        Self {
            name,
            fingerprint,
        }
    }
}

