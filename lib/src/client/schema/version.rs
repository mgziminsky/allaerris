use std::fmt::Display;

use github::models::AssetId;
use serde::{Deserialize, Serialize};
use url::Url;

use super::ProjectId;
use crate::{checked_types::PathScoped, client::service_id::svc_id_impl, config::ModLoader};

svc_id_impl! {
    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
    #[serde(rename_all = "lowercase")]
    pub enum VersionId {
        Forge(u64),
        Modrinth(String),
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
