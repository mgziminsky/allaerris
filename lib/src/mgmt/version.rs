use crate::client::schema::{self, ProjectId};

pub type VersionSet = std::collections::HashSet<VersionByProject>;

#[derive(Debug, Clone)]
pub(super) struct VersionByProject(schema::Version);


impl From<schema::Version> for VersionByProject {
    fn from(val: schema::Version) -> Self {
        Self(val)
    }
}
impl From<VersionByProject> for schema::Version {
    fn from(val: VersionByProject) -> Self {
        val.0
    }
}
/// Required to allow set lookups by `path` only
impl std::borrow::Borrow<ProjectId> for VersionByProject {
    fn borrow(&self) -> &ProjectId {
        &self.0.project_id
    }
}
impl AsRef<schema::Version> for VersionByProject {
    fn as_ref(&self) -> &schema::Version {
        self
    }
}
impl std::ops::Deref for VersionByProject {
    type Target = schema::Version;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl Eq for VersionByProject {}
impl PartialEq for VersionByProject {
    fn eq(&self, other: &Self) -> bool {
        self.0.project_id == other.0.project_id
    }
}
impl std::hash::Hash for VersionByProject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.project_id.hash(state);
    }
}
