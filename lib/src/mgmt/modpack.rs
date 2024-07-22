pub mod forge;
pub mod modrinth;
mod version;

use std::{collections::HashMap, io::Read, path::Path};

use ::modrinth::{
    apis::version_files_api::VersionsFromHashesParams,
    models::{hash_list::Algorithm, HashList},
};
use anyhow::anyhow;
use tokio::fs::File;
use zip::ZipArchive;

use self::{
    forge::ModpackManifest,
    modrinth::{IndexFile, ModpackIndex, PackDependency},
};
use super::{cache, events::EventSouce, version::VersionSet};
use crate::{
    checked_types::{PathScoped, PathScopedRef},
    client::schema::{ProjectId, Version, VersionId},
    config::{profile::ProfileData, ModLoader, VersionedProject},
    Client, ProfileManager, Result,
};

type PackArchive = ZipArchive<std::fs::File>;


impl ProfileManager {
    pub(super) async fn load_modpack(
        &self,
        client: &Client,
        pack: &impl VersionedProject,
        data: &ProfileData,
    ) -> Result<(Version, ModpackData)> {
        let mut pack_version = if let Some(vid) = pack.version() {
            client.get_version(vid).await?
        } else {
            client
                .get_latest(pack.project(), Some(&data.game_version), Some(data.loader))
                .await?
        };
        let cache_path = cache::version_path(&pack_version, PathScopedRef::new("modpacks").ok());
        let Some(sha1) = self.dl_version(&pack_version, &cache_path).await else {
            return Err(anyhow!("Modpack download failed").into());
        };
        pack_version.sha1.replace(sha1);

        self.read_pack(client, &cache_path).await.map(|p| (pack_version, p))
    }

    async fn read_pack(&self, client: &Client, path: &Path) -> Result<ModpackData> {
        let mut zip = PackArchive::new(File::open(path).await?.into_std().await).map_err(anyhow::Error::new)?;

        macro_rules! parse {
            (|$name:ident = $file:literal| $block:expr) => {
                let pack = if let Ok($name) = zip.by_name($file) {
                    let data = $block;
                    Some(data)
                } else {
                    None
                };
                // Need separate block for return as a workaround for this: https://github.com/rust-lang/rust/issues/92985
                if let Some((mods, overrides_prefix)) = pack {
                    return Ok(ModpackData {
                        archive: zip,
                        overrides_prefix,
                        mods,
                    });
                }
            };
        }
        macro_rules! read_json {
            ($var:ident) => {
                serde_json::from_reader($var).map_err(anyhow::Error::new)?
            };
        }
        parse!(|index = "modrinth.index.json"| {
            let (known, unknown) = self.parse_modrinth(
                client.as_modrinth().ok_or(anyhow!("Modrinth modpack found, but no Modrinth client available"))?,
                read_json!(index),
            ).await?;
            (PackMods::Modrinth { known, unknown }, PathScoped::new("overrides").unwrap())
        });
        parse!(|manifest = "manifest.json"| {
            let (mods, prefix) = parse_forge(read_json!(manifest));
            (PackMods::Forge(mods), prefix)
        });

        Err(anyhow!("Invalid or unsupported modpack").into())
    }

    async fn parse_modrinth(&self, client: &::modrinth::ApiClient, index: ModpackIndex) -> Result<(VersionSet, Vec<IndexFile>)> {
        match index {
            ModpackIndex::V1 {
                files,
                name: pack_name,
                mut deps,
                ..
            } => {
                let game_version = deps.remove(&PackDependency::Minecraft);
                let loaders: Vec<_> = deps
                    .into_keys()
                    .map(|l| match l {
                        PackDependency::Minecraft => unreachable!(),
                        PackDependency::Forge => ModLoader::Forge,
                        PackDependency::Neoforge => ModLoader::NeoForge,
                        PackDependency::FabricLoader => ModLoader::Fabric,
                        PackDependency::QuiltLoader => ModLoader::Quilt,
                    })
                    .collect();

                let mut versions = VersionSet::new();
                let mut pending = vec![];

                for f in &files {
                    let path = match f.path_scoped() {
                        Ok(p) => p,
                        Err(e) => {
                            self.send_err(
                                anyhow::Error::new(e)
                                    .context(format!("Invalid modpack file: `{}`", f.path.display()))
                                    .into(),
                            );
                            continue;
                        },
                    };
                    let Ok((pid, vid)) = f.index_version() else {
                        pending.push(f);
                        continue;
                    };
                    versions.insert(
                        Version {
                            id: vid,
                            project_id: pid,
                            title: format!("{pack_name} - {}", path.display()),
                            download_url: f.downloads.first().cloned(),
                            filename: path.to_owned(),
                            length: f.file_size,
                            date: Default::default(),
                            sha1: Some(f.hashes.sha1.clone()),
                            deps: Default::default(),
                            game_versions: game_version.iter().cloned().collect(),
                            loaders: loaders.clone(),
                        }
                        .into(),
                    );
                }
                let fetched = client
                    .version_files()
                    .versions_from_hashes(&VersionsFromHashesParams {
                        hash_list: Some(&HashList {
                            hashes: pending.iter().map(|f| f.hashes.sha1.clone()).collect(),
                            algorithm: Algorithm::Sha1,
                        }),
                    })
                    .await?;
                let unknown = pending
                    .into_iter()
                    .filter(|f| !fetched.contains_key(&f.hashes.sha1))
                    .cloned()
                    .collect();
                versions.extend(fetched.into_values().map(Version::from).map(Into::into));
                Ok((versions, unknown))
            },
        }
    }
}

fn parse_forge(manifest: ModpackManifest) -> (HashMap<ProjectId, VersionId>, PathScoped) {
    match manifest {
        ModpackManifest::V1 { files, overrides, .. } => (
            files
                .into_iter()
                .map(|f| (ProjectId::Forge(f.project_id), VersionId::Forge(f.file_id)))
                .collect(),
            overrides,
        ),
    }
}


#[derive(Debug)]
pub enum PackMods {
    Modrinth { known: VersionSet, unknown: Vec<IndexFile> },
    Forge(HashMap<ProjectId, VersionId>),
}

#[derive(Debug)]
pub struct ModpackData {
    archive: PackArchive,
    overrides_prefix: PathScoped,
    pub mods: PackMods,
}

impl ModpackData {
    pub fn visit_overrides(&mut self, mut cb: impl FnMut(&PathScopedRef, ZipFile)) {
        let zip = &mut self.archive;
        for idx in 0..zip.len() {
            let Ok(file) = zip.by_index(idx) else {
                continue;
            };
            if !file.is_file() {
                // Do we need to handle symlinks?
                continue;
            }
            let Some(Ok(path)) = file.enclosed_name().map(PathScoped::new) else {
                continue;
            };
            if !path.starts_with(&self.overrides_prefix) {
                continue;
            }
            let path = path.remove_prefix(path.iter().next().unwrap());

            cb(path, ZipFile(file));
        }
    }
}

/// Unit wrapper around lib `ZipFile` only exposing [`Read`] trait
pub struct ZipFile<'a>(zip::read::ZipFile<'a>);
impl<'a> Read for ZipFile<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}
