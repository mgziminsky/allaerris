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

/// Hash algorithems Possible enum values:  * 1 = Sha1  * 2 = Md5
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum HashAlgo {
    #[default]
    #[serde(rename = "1")]
    Sha1,
    #[serde(rename = "2")]
    Md5,
}

impl std::fmt::Display for HashAlgo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Sha1 => "1",
            Self::Md5 => "2",
        })
    }
}
