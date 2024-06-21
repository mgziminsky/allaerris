use serde::{Deserialize, Serialize};

use crate::client::schema::{self, Project, ProjectId};

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
}

impl Eq for Mod {}
impl PartialEq for Mod {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::hash::Hash for Mod {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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
            id: proj.id,
            slug: proj.slug,
            name: proj.name,
        }
    }
}
