#![allow(dead_code)]

use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;
use url::Url;

use crate::{
    checked_types::{PathScopeError, PathScopedRef},
    client::schema::{ProjectId, VersionId},
};


#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ModpackIndex {
    #[serde(rename_all = "camelCase")]
    V1 {
        format_version: super::version::Version<1>,
        game: Game,
        version_id: String,
        name: String,
        summary: Option<String>,
        files: Vec<IndexFile>,
        #[serde(rename = "dependencies")]
        deps: HashMap<PackDependency, String>,
    },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexFile {
    pub path: PathBuf,
    pub hashes: Hashes,
    pub downloads: Vec<Url>,
    pub env: Option<PackEnv>,
    pub file_size: u64,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Minecraft,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Hashes {
    pub sha1: String,
    pub sha512: String,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PackDependency {
    Minecraft,
    Forge,
    Neoforge,
    FabricLoader,
    QuiltLoader,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PackEnv {
    pub client: DependencyType,
    pub server: DependencyType,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Required,
    Optional,
    Unsupported,
}


impl IndexFile {
    /// Attempt to extract the project/version ids from a modrinth download url
    pub fn index_version(&self) -> Result<(ProjectId, VersionId), ()> {
        const DL_PREFIX: &str = "https://cdn.modrinth.com/data/";
        let Some(dl) = self.downloads.iter().find(|url| url.as_str().starts_with(DL_PREFIX)) else {
            return Err(());
        };
        let mut path = dl.path_segments().unwrap();
        macro_rules! nth_1 {
            ($ty:expr) => {
                path.nth(1).map(ToString::to_string).map($ty).ok_or(())
            };
        }
        let project_id = nth_1!(ProjectId::Modrinth)?;
        let id = nth_1!(VersionId::Modrinth)?;
        Ok((project_id, id))
    }

    pub fn path_scoped(&self) -> Result<&PathScopedRef, PathScopeError> {
        PathScopedRef::new(&self.path)
    }
}