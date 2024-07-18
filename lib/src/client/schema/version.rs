use std::fmt::Display;

use github::models::AssetId;
use serde::{Deserialize, Serialize};
use url::Url;

use super::ProjectId;
use crate::{
    checked_types::PathScoped,
    client::{service_id::svc_id_type, ServiceId},
    config::ModLoader,
    ErrorKind, Result,
};


svc_id_type! {
    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
    #[serde(rename_all = "lowercase")]
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


impl VersionIdSvcType for VersionId {
    #[inline]
    fn get_forge(&self) -> Result<<VersionId as ServiceId>::ForgeT> {
        if let Self::Forge(v) = self {
            Ok(*v)
        } else {
            Err(crate::ErrorKind::WrongService)?
        }
    }

    #[inline]
    fn get_modrinth(&self) -> Result<&str> {
        if let Self::Modrinth(v) = self {
            Ok(v)
        } else {
            Err(crate::ErrorKind::WrongService)?
        }
    }

    #[inline]
    fn get_github(&self) -> Result<<VersionId as ServiceId>::GithubT> {
        if let Self::Github(v) = self {
            Ok(*v)
        } else {
            Err(crate::ErrorKind::WrongService)?
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
