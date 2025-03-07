/*
 * CurseForge API
 *
 * HTTP API for CurseForge
 *
 * The version of the OpenAPI document: 1.0.240719
 * 
 * Generated by: https://openapi-generator.tech
 */

#[allow(unused_imports)]
use crate::models;

/// Status of the file Possible enum values:  * 1 = Processing  * 2 = ChangesRequired  * 3 = UnderReview  * 4 = Approved  * 5 = Rejected  * 6 = MalwareDetected  * 7 = Deleted  * 8 = Archived  * 9 = Testing  * 10 = Released  * 11 = ReadyForReview  * 12 = Deprecated  * 13 = Baking  * 14 = AwaitingPublishing  * 15 = FailedPublishing 
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum FileStatus {
    #[default]
    Processing = 1,
    ChangesRequired = 2,
    UnderReview = 3,
    Approved = 4,
    Rejected = 5,
    MalwareDetected = 6,
    Deleted = 7,
    Archived = 8,
    Testing = 9,
    Released = 10,
    ReadyForReview = 11,
    Deprecated = 12,
    Baking = 13,
    AwaitingPublishing = 14,
    FailedPublishing = 15,
}

impl std::fmt::Display for FileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Processing => "1",
            Self::ChangesRequired => "2",
            Self::UnderReview => "3",
            Self::Approved => "4",
            Self::Rejected => "5",
            Self::MalwareDetected => "6",
            Self::Deleted => "7",
            Self::Archived => "8",
            Self::Testing => "9",
            Self::Released => "10",
            Self::ReadyForReview => "11",
            Self::Deprecated => "12",
            Self::Baking => "13",
            Self::AwaitingPublishing => "14",
            Self::FailedPublishing => "15",
        })
    }
}

