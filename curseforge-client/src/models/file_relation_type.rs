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

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum FileRelationType {
    #[default]
    #[serde(rename = "1")]
    EmbeddedLibrary,
    #[serde(rename = "2")]
    OptionalDependency,
    #[serde(rename = "3")]
    RequiredDependency,
    #[serde(rename = "4")]
    Tool,
    #[serde(rename = "5")]
    Incompatible,
    #[serde(rename = "6")]
    Include,
}

impl std::fmt::Display for FileRelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmbeddedLibrary => "1",
            Self::OptionalDependency => "2",
            Self::RequiredDependency => "3",
            Self::Tool => "4",
            Self::Incompatible => "5",
            Self::Include => "6",
        })
    }
}

