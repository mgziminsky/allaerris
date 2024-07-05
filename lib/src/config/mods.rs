use serde::{Deserialize, Serialize};

use super::ProjectWithVersion;
use crate::client::schema::{self, Project, ProjectId, VersionId};

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
}

impl Mod {
    /// The [project id](ProjectId) of this mod
    pub fn id(&self) -> &ProjectId {
        self.id.project()
    }

    /// The [version id](VersionId) of this mod if present
    pub fn version(&self) -> Option<&VersionId> {
        self.id.version()
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

impl From<schema::Mod> for Mod {
    fn from(m: schema::Mod) -> Self {
        m.0.into()
    }
}
impl From<Project> for Mod {
    fn from(proj: Project) -> Self {
        Self {
            id: proj.id.into(),
            slug: proj.slug,
            name: proj.name,
        }
    }
}
