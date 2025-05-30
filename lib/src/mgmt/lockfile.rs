use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::{
    Result, StdResult,
    checked_types::PathScoped,
    client::schema::{self, ProjectId, VersionId},
    config::{ModLoader, Profile, ProjectWithVersion, VersionedProject, profile},
    fs_util::{FsUtil, FsUtils},
};

crate::cow::cow!(LockedMod);

pub type PathHashes = BTreeMap<PathScoped, String>;
const FILENAME: &str = concat!(profile::consts!(FILENAME), ".lock");

fn cmp_files(a: &LockedMod, b: &LockedMod) -> std::cmp::Ordering {
    a.file.cmp(&b.file)
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LockFile {
    pub game_version: String,
    pub loader: ModLoader,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack: Option<LockedPack>,

    #[serde(default, skip_serializing_if = "Vec::is_empty", deserialize_with = "deduped_mods")]
    pub mods: Vec<LockedMod>,

    #[serde(default, skip_serializing_if = "PathHashes::is_empty")]
    pub other: PathHashes,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outdated: Vec<LockedMod>,
}

impl LockFile {
    pub fn file_path(profile_path: impl AsRef<Path>) -> PathBuf {
        profile_path.as_ref().join(FILENAME)
    }

    pub fn sort(&mut self) {
        self.mods.sort_unstable_by(cmp_files);
        self.outdated.sort_unstable_by(cmp_files);
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
impl Profile {
    /// Returns the basic info about the installed versions of the mods and
    /// modpack for this [`Profile`].
    ///
    /// # Errors
    ///
    /// This function will return an error if reading the lockfile fails.
    pub async fn installed(&self) -> Result<Vec<LockedMod>> {
        let LockFile { mut mods, pack, .. } = LockFile::load(self.path()).await?;
        if let Some(pack) = pack {
            mods.push(pack.data);
        }
        Ok(mods)
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
    fn project(&self) -> &ProjectId {
        self.id.project()
    }

    fn version(&self) -> Option<&VersionId> {
        self.id.version()
    }
}

impl From<schema::Version> for LockedMod {
    fn from(v: schema::Version) -> Self {
        LockedMod {
            id: LockedId {
                project: v.project_id,
                version: v.id,
            },
            sha1: v.sha1.unwrap_or_default(),
            file: v.filename,
        }
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
impl From<LockedId> for ProjectId {
    fn from(lid: LockedId) -> Self {
        lid.project
    }
}
impl From<LockedId> for VersionId {
    fn from(lid: LockedId) -> Self {
        lid.version
    }
}
impl VersionedProject for LockedId {
    fn project(&self) -> &ProjectId {
        &self.project
    }

    fn version(&self) -> Option<&VersionId> {
        Some(&self.version)
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


fn deduped_mods<'de, D>(de: D) -> StdResult<Vec<LockedMod>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct ModSetVisitor;
    impl<'de> serde::de::Visitor<'de> for ModSetVisitor {
        type Value = Vec<LockedMod>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A list of locked mods unique on project id")
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            #[derive(Deserialize)]
            #[serde(transparent)]
            struct ById(LockedMod);
            impl Eq for ById {}
            impl PartialEq for ById {
                fn eq(&self, other: &Self) -> bool {
                    self.0.project() == other.0.project()
                }
            }
            impl std::hash::Hash for ById {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    self.0.project().hash(state);
                }
            }


            let mut mods = HashSet::<ById>::new();
            while let Some(lm) = seq.next_element()? {
                if mods.contains(&lm) {
                    return Err(serde::de::Error::custom(format!("Duplicate Mod: {}", lm.0.project())));
                }
                mods.insert(lm);
            }
            Ok(mods.into_iter().map(|m| m.0).collect())
        }
    }

    de.deserialize_seq(ModSetVisitor)
}
