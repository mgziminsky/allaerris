use std::fmt::Display;

use serde::{Deserialize, Serialize};
use url::Url;

use super::{Author, License};
use crate::{
    client::{service_id::svc_id_impl, ServiceId},
    ErrorKind, Result,
};


svc_id_impl! {
    /// The [client](crate::client) specific project id types
    #[derive(Deserialize, Serialize, Debug, Clone, Eq, Hash)]
    pub enum ProjectId {
        Forge(u64),
        Modrinth(String),
        Github((String, String)),
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub downloads: u64,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub icon: Option<Url>,
    pub authors: Vec<Author>,
    pub categories: Vec<String>,
    pub license: Option<License>,
    pub website: Option<Url>,
    pub source_url: Option<Url>,
}

impl Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ProjectId::Forge(id) => f.write_fmt(format_args!("{id}")),
            ProjectId::Modrinth(id) => f.write_str(id),
            ProjectId::Github((owner, name)) => f.write_fmt(format_args!("{}/{}", owner.to_lowercase(), name.to_lowercase())),
        }
    }
}

impl<T: AsProjectId> PartialEq<T> for ProjectId {
    fn eq(&self, other: &T) -> bool {
        match self {
            ProjectId::Forge(id) => other.try_as_forge().is_ok_and(|oid| *id == oid),
            ProjectId::Modrinth(id) => other.try_as_modrinth().is_ok_and(|other| id == other),
            ProjectId::Github((owner, repo)) => other
                .try_as_github()
                .is_ok_and(|(owner_other, repo_other)| owner == owner_other && repo == repo_other),
        }
    }
}
impl<T: AsProjectId> PartialOrd<T> for ProjectId {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        match self {
            ProjectId::Forge(id) => other.try_as_forge().ok().and_then(|oid| id.partial_cmp(&oid)),
            ProjectId::Modrinth(id) => other.try_as_modrinth().ok().map(|other| id.as_str().cmp(other)),
            ProjectId::Github((owner, repo)) => other
                .try_as_github()
                .ok()
                .and_then(|other| (owner.as_str(), repo.as_str()).partial_cmp(&other)),
        }
    }
}


pub trait AsProjectId: Sync {
    fn try_as_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT>;
    fn try_as_modrinth(&self) -> Result<&str>;
    fn try_as_github(&self) -> Result<(&str, &str)>;
}

impl AsProjectId for ProjectId {
    fn try_as_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        self.as_forge().copied()
    }

    fn try_as_modrinth(&self) -> Result<&str> {
        self.as_modrinth().map(AsRef::as_ref)
    }

    fn try_as_github(&self) -> Result<(&str, &str)> {
        self.as_github().map(|(o, r)| (o.as_str(), r.as_str()))
    }
}

impl AsProjectId for str {
    fn try_as_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        self.parse().map_err(|_| ErrorKind::InvalidIdentifier.into())
    }

    fn try_as_modrinth(&self) -> Result<&str> {
        Ok(self)
    }

    fn try_as_github(&self) -> Result<(&str, &str)> {
        self.split_once('/').ok_or(ErrorKind::InvalidIdentifier.into())
    }
}

impl AsProjectId for String {
    fn try_as_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        self.as_str().try_as_forge()
    }

    fn try_as_modrinth(&self) -> Result<&str> {
        self.as_str().try_as_modrinth()
    }

    fn try_as_github(&self) -> Result<(&str, &str)> {
        self.as_str().try_as_github()
    }
}

impl<T: AsProjectId + ?Sized> AsProjectId for &T {
    fn try_as_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        (*self).try_as_forge()
    }

    fn try_as_modrinth(&self) -> Result<&str> {
        (*self).try_as_modrinth()
    }

    fn try_as_github(&self) -> Result<(&str, &str)> {
        (*self).try_as_github()
    }
}
