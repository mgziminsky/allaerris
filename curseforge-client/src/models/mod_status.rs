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

/// Current mod status Possible enum values:  * 1 = New  * 2 = ChangesRequired  * 3 = UnderSoftReview  * 4 = Approved  * 5 = Rejected  * 6 = ChangesMade  * 7 = Inactive  * 8 = Abandoned  * 9 = Deleted  * 10 = UnderReview 
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ModStatus {
    #[default]
    #[serde(rename = "1")]
    Variant1,
    #[serde(rename = "2")]
    Variant2,
    #[serde(rename = "3")]
    Variant3,
    #[serde(rename = "4")]
    Variant4,
    #[serde(rename = "5")]
    Variant5,
    #[serde(rename = "6")]
    Variant6,
    #[serde(rename = "7")]
    Variant7,
    #[serde(rename = "8")]
    Variant8,
    #[serde(rename = "9")]
    Variant9,
    #[serde(rename = "10")]
    Variant10,
}

impl std::fmt::Display for ModStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Variant1 => "1",
            Self::Variant2 => "2",
            Self::Variant3 => "3",
            Self::Variant4 => "4",
            Self::Variant5 => "5",
            Self::Variant6 => "6",
            Self::Variant7 => "7",
            Self::Variant8 => "8",
            Self::Variant9 => "9",
            Self::Variant10 => "10",
        })
    }
}

