use std::fmt::Display;

use serde::{Deserialize, Serialize};
use url::Url;

use super::{Author, License};
use crate::{
    client::{service_id::svc_id_type, ServiceId},
    ErrorKind, Result,
};


svc_id_type! {
    /// The [client](crate::client) specific project id types
    #[derive(Debug, Clone, Eq, Hash, Ord)]
    pub enum ProjectId {
        Forge(u64),
        Modrinth(String = &str),
        Github((String, String) = (&str, &str)),
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    #[default]
    Mod,
    ModPack,
    ResourcePack,
    DataPack,
    Shader,
}
impl ProjectType {
    pub fn install_dir(&self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::ModPack => "",
            Self::ResourcePack => "resourcepacks",
            Self::DataPack => "datapacks",
            Self::Shader => "shaderpacks",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub project_type: ProjectType,
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
        match self {
            Self::Forge(id) => f.write_fmt(format_args!("{id}")),
            Self::Modrinth(id) => f.write_str(id),
            Self::Github((owner, name)) => f.write_fmt(format_args!("{}/{}", owner.to_lowercase(), name.to_lowercase())),
        }
    }
}

impl<T: ProjectIdSvcType + ?Sized> PartialEq<T> for ProjectId {
    fn eq(&self, other: &T) -> bool {
        match self {
            Self::Forge(id) => other.get_forge().is_ok_and(|oid| *id == oid),
            Self::Modrinth(id) => other.get_modrinth().is_ok_and(|other| id == other),
            Self::Github((owner, repo)) => other
                .get_github()
                .is_ok_and(|(owner_other, repo_other)| owner == owner_other && repo == repo_other),
        }
    }
}
impl<T: ProjectIdSvcType + ?Sized> PartialOrd<T> for ProjectId {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        match self {
            Self::Forge(id) => other.get_forge().ok().and_then(|oid| id.partial_cmp(&oid)),
            Self::Modrinth(id) => other.get_modrinth().ok().map(|other| id.as_str().cmp(other)),
            Self::Github((owner, repo)) => other
                .get_github()
                .ok()
                .and_then(|other| (owner.as_str(), repo.as_str()).partial_cmp(&other)),
        }
    }
}

impl ProjectIdSvcType for ProjectId {
    #[inline]
    fn get_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        if let Self::Forge(v) = self {
            Ok(*v)
        } else {
            Err(crate::ErrorKind::WrongService(self.to_string()))?
        }
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        if let Self::Modrinth(v) = self {
            Ok(v)
        } else {
            Err(crate::ErrorKind::WrongService(self.to_string()))?
        }
    }

    #[inline]
    fn get_github(&self) -> Result<(&str, &str)> {
        if let Self::Github((o, r)) = self {
            Ok((o, r))
        } else {
            Err(crate::ErrorKind::WrongService(self.to_string()))?
        }
    }
}

impl ProjectIdSvcType for u64 {
    #[inline]
    fn get_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        Ok(*self)
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        Err(ErrorKind::InvalidIdentifier.into())
    }

    #[inline]
    fn get_github(&self) -> Result<(&str, &str)> {
        Err(ErrorKind::InvalidIdentifier.into())
    }
}

impl ProjectIdSvcType for str {
    #[inline]
    fn get_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        self.parse().map_err(|_| ErrorKind::InvalidIdentifier.into())
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        Ok(self)
    }

    #[inline]
    fn get_github(&self) -> Result<(&str, &str)> {
        self.split_once('/').ok_or(ErrorKind::InvalidIdentifier.into())
    }
}

impl ProjectIdSvcType for String {
    #[inline]
    fn get_forge(&self) -> Result<<ProjectId as ServiceId>::ForgeT> {
        self.as_str().get_forge()
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        self.as_str().get_modrinth()
    }

    #[inline]
    fn get_github(&self) -> Result<(&str, &str)> {
        self.as_str().get_github()
    }
}
