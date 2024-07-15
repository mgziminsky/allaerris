use std::path::{Path, PathBuf};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    checked_types::PathScoped,
    client::schema::{ProjectId, VersionId},
    config::{profile, ModLoader, ProjectWithVersion},
    fs_util::{FsUtil, FsUtils},
    Result, StdResult,
};

const FILENAME: &str = concat!(profile::consts!(FILENAME), ".lock");


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LockFile {
    pub game_version: String,
    pub loader: ModLoader,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mods: Vec<LockedMod>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<PathScoped>,
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

impl LockedMod {
    pub fn project(&self) -> &ProjectId {
        self.id.project()
    }

    pub fn version(&self) -> &VersionId {
        self.id.version()
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

    fn project(&self) -> &ProjectId {
        &self.project
    }

    fn version(&self) -> &VersionId {
        &self.version
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

crate::cow::cow!(LockedMod);
