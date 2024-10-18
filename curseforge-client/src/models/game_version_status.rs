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

/// Possible enum values:  * 1 = Approved  * 2 = Deleted  * 3 = New 
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum GameVersionStatus {
    #[default]
    Approved = 1,
    Deleted = 2,
    New = 3,
}

impl std::fmt::Display for GameVersionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Approved => "1",
            Self::Deleted => "2",
            Self::New => "3",
        })
    }
}

