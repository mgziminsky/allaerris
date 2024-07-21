use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    checked_types::PathScoped,
    client::schema::{ProjectId, VersionId},
    config::{profile, ModLoader, ProjectWithVersion, VersionedProject},
    fs_util::{FsUtil, FsUtils},
    Result, StdResult,
};

pub type PathHashes = BTreeMap<PathScoped, String>;
const FILENAME: &str = concat!(profile::consts!(FILENAME), ".lock");


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LockFile {
    pub game_version: String,
    pub loader: ModLoader,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack: Option<LockedPack>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<LockedMod>,

    #[serde(default, skip_serializing_if = "PathHashes::is_empty")]
    pub other: PathHashes,
}

impl LockFile {
    #[inline]
    pub fn file_path(profile_path: impl AsRef<Path>) -> PathBuf {
        profile_path.as_ref().join(FILENAME)
    }

    /// Load the [lock file](LockFile) located at `profile_path` or return a
    /// [default](LockFile::default) if the file does not exist.
    pub async fn load(profile_path: impl AsRef<Path>) -> Result<Self> {
        let lock_path = &Self::file_path(profile_path);
        let lockfile = if lock_path.exists() {
            FsUtil::load_file(lock_path).await?
        } else {
            LockFile::default()
        };
        Ok(lockfile)
    }

    pub async fn save(&self, profile_path: impl AsRef<Path>) -> Result<()> {
        let path = &Self::file_path(profile_path);
        let res = FsUtil::save_file(&self, path).await;
        if res.is_err() {
            let _ = tokio::fs::remove_file(path).await;
        }
        res
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LockedMod {
    #[serde(flatten)]
    pub id: LockedId,
    pub file: PathScoped,
    pub sha1: String,
}

impl VersionedProject for LockedMod {
    #[inline]
    fn project(&self) -> &ProjectId {
        &self.id.project
    }

    #[inline]
    fn version(&self) -> Option<&VersionId> {
        Some(&self.id.version)
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "ProjectWithVersion", into = "ProjectWithVersion")]
pub struct LockedId {
    pub(super) project: ProjectId,
    pub(super) version: VersionId,
}

impl LockedId {
    #[inline]
    pub fn new(project: ProjectId, version: VersionId) -> StdResult<Self, anyhow::Error> {
        ProjectWithVersion::new(project, Some(version))?.try_into()
    }
}

impl TryFrom<ProjectWithVersion> for LockedId {
    type Error = anyhow::Error;

    fn try_from(val: ProjectWithVersion) -> StdResult<Self, Self::Error> {
        if let Some(version) = val.version {
            Ok(Self {
                project: val.project,
                version,
            })
        } else {
            Err(anyhow!("missing `version`"))
        }
    }
}
impl From<LockedId> for ProjectWithVersion {
    fn from(lid: LockedId) -> Self {
        Self {
            project: lid.project,
            version: Some(lid.version),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockedPack {
    #[serde(flatten)]
    pub data: LockedMod,

    #[serde(default, skip_serializing_if = "PathHashes::is_empty")]
    pub overrides: PathHashes,
}

impl LockedPack {
    pub fn new(data: LockedMod) -> Self {
        Self {
            data,
            overrides: PathHashes::new(),
        }
    }
}

impl VersionedProject for LockedPack {
    #[inline]
    fn project(&self) -> &ProjectId {
        self.data.project()
    }

    #[inline]
    fn version(&self) -> Option<&VersionId> {
        self.data.version()
    }
}

impl std::ops::Deref for LockedPack {
    type Target = LockedMod;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl std::ops::DerefMut for LockedPack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}


crate::cow::cow!(LockedMod);
