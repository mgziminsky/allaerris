use serde::{Deserialize, Serialize};

use super::{ProjectWithVersion, project_with_version::VersionedProject};
use crate::client::schema::{Project, ProjectId, ProjectType, VersionId};

/// The basic data needed to lookup and install a particular mod from one of the
/// [supported clients](crate::client)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    /// The [client](crate::client) specific id of the mod
    #[serde(flatten)]
    pub id: ProjectWithVersion,

    /// The project slug for this mod as typically seen in the URL
    pub slug: String,

    /// The local name of this mod. May not match actual project name from the
    /// [client](crate::Client)
    pub name: String,

    /// The [type](ProjectType) of this mod
    pub project_type: ProjectType,

    /// If `true`, will prevent this mod from being installed by a modpack
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub exclude: bool,
}

impl VersionedProject for Mod {
    #[inline]
    fn project(&self) -> &ProjectId {
        &self.id.project
    }

    #[inline]
    fn version(&self) -> Option<&VersionId> {
        self.id.version.as_ref()
    }
}

impl Eq for Mod {}
impl PartialEq for Mod {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for Mod {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.project.hash(state);
    }
}

impl From<Project> for Mod {
    fn from(proj: Project) -> Self {
        Self {
            id: proj.id.into(),
            slug: proj.slug,
            name: proj.name,
            exclude: false,
            project_type: proj.project_type,
        }
    }
}
