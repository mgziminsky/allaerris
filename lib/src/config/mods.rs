use serde::{Deserialize, Serialize};

use crate::client::schema::ProjectId;

/// The basic data needed to lookup and install a particular mod from one of the
/// [supported clients](crate::client)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    /// The [client](crate::client) specific id of the mod
    #[serde(flatten)]
    pub id: ProjectId,

    /// The project slug for this mod as typically seen in the URL
    pub slug: String,

    /// The local name of this mod. May not match actual project name from the
    /// [client](crate::Client)
    pub name: String,

    /// If set, overrides the path the mod should be installed to instead of
    /// using the path specified by the client
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl PartialEq for Mod {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
