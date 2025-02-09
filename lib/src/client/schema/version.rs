use std::fmt::Display;

use github::models::AssetId;
use url::Url;

use super::ProjectId;
use crate::{
    checked_types::PathScoped,
    client::{service_id::svc_id_type, ServiceId},
    config::{ModLoader, VersionedProject},
    ErrorKind, Result,
};


svc_id_type! {
    #[derive(Debug, Clone, Eq, Hash)]
    pub enum VersionId {
        Forge(u64),
        Modrinth(String = &str),
        Github(AssetId),
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub id: VersionId,
    pub project_id: ProjectId,
    pub title: String,
    pub download_url: Option<Url>,
    pub filename: PathScoped,
    pub length: u64,
    pub date: String,
    pub sha1: Option<String>,
    pub deps: Vec<Dependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<ModLoader>,
}

impl VersionedProject for Version {
    fn project(&self) -> &ProjectId {
        &self.project_id
    }

    fn version(&self) -> Option<&VersionId> {
        Some(&self.id)
    }
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub project_id: ProjectId,
    pub id: Option<VersionId>,
    pub dep_type: DependencyType,
}

#[derive(Debug, Clone, Copy)]
pub enum DependencyType {
    Required,
    Optional,
    Other,
}


impl Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val: &dyn Display = match self {
            Self::Forge(id) => id,
            Self::Modrinth(id) => id,
            Self::Github(id) => id,
        };
        write!(f, "{val}")
    }
}
impl<T: VersionIdSvcType> PartialEq<T> for VersionId {
    fn eq(&self, other: &T) -> bool {
        match self {
            VersionId::Forge(id) => other.get_forge().is_ok_and(|oid| *id == oid),
            VersionId::Modrinth(id) => other.get_modrinth().is_ok_and(|other| id == other),
            VersionId::Github(id) => other.get_github().is_ok_and(|oid| *id == oid),
        }
    }
}
impl<T: VersionIdSvcType> PartialOrd<T> for VersionId {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        match self {
            VersionId::Forge(id) => other.get_forge().ok().and_then(|oid| id.partial_cmp(&oid)),
            VersionId::Modrinth(id) => other.get_modrinth().ok().map(|other| id.as_str().cmp(other)),
            VersionId::Github(id) => other.get_github().ok().and_then(|oid| id.partial_cmp(&oid)),
        }
    }
}

impl VersionIdSvcType for VersionId {
    #[inline]
    fn get_forge(&self) -> Result<<VersionId as ServiceId>::ForgeT> {
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
    fn get_github(&self) -> Result<<VersionId as ServiceId>::GithubT> {
        if let Self::Github(v) = self {
            Ok(*v)
        } else {
            Err(crate::ErrorKind::WrongService(self.to_string()))?
        }
    }
}

impl VersionIdSvcType for u64 {
    #[inline]
    fn get_forge(&self) -> Result<<VersionId as ServiceId>::ForgeT> {
        Ok(*self)
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        Err(ErrorKind::InvalidIdentifier.into())
    }

    #[inline]
    fn get_github(&self) -> Result<<VersionId as ServiceId>::GithubT> {
        Err(ErrorKind::InvalidIdentifier.into())
    }
}

impl VersionIdSvcType for <VersionId as ServiceId>::GithubT {
    #[inline]
    fn get_forge(&self) -> Result<<VersionId as ServiceId>::ForgeT> {
        Err(ErrorKind::InvalidIdentifier.into())
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        Err(ErrorKind::InvalidIdentifier.into())
    }

    #[inline]
    fn get_github(&self) -> Result<<VersionId as ServiceId>::GithubT> {
        Ok(*self)
    }
}

impl VersionIdSvcType for str {
    #[inline]
    fn get_forge(&self) -> Result<<VersionId as ServiceId>::ForgeT> {
        self.parse().map_err(|_| ErrorKind::InvalidIdentifier.into())
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        Ok(self)
    }

    #[inline]
    fn get_github(&self) -> Result<<VersionId as ServiceId>::GithubT> {
        self.parse::<u64>().map(Into::into).map_err(|_| ErrorKind::InvalidIdentifier.into())
    }
}

impl VersionIdSvcType for String {
    #[inline]
    fn get_forge(&self) -> Result<<VersionId as ServiceId>::ForgeT> {
        self.as_str().get_forge()
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        self.as_str().get_modrinth()
    }

    #[inline]
    fn get_github(&self) -> Result<<VersionId as ServiceId>::GithubT> {
        self.as_str().get_github()
    }
}
