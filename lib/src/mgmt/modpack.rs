pub mod forge;
pub mod modrinth;
mod version;

use std::collections::HashMap;

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
    checked_types::PathScopedRef,
    client::schema::{ProjectId, Version, VersionId},
    config::{profile::ProfileData, ModLoader, Modpack},
    Client, ProfileManager, Result,
};

type PackArchive = ZipArchive<std::fs::File>;

#[derive(Debug)]
pub struct ModpackData {
    archive: PackArchive,
    pub mods: PackMods,
}

#[derive(Debug)]
pub enum PackMods {
    Modrinth { known: VersionSet, unknown: Vec<IndexFile> },
    Forge(HashMap<ProjectId, VersionId>),
}

impl ProfileManager {
    pub(super) async fn load_modpack(&self, client: &Client, modpack: &Modpack, data: &ProfileData) -> Result<ModpackData> {
        let pack_version = client.get_latest(modpack.id(), Some(&data.game_version), Some(data.loader)).await?;
        let cache_path = cache::version_path(&pack_version, Some(PathScopedRef::new("modpacks").unwrap()));
        self.dl_version(pack_version, &cache_path).await.unwrap();

        let mut zip = PackArchive::new(File::open(cache_path).await?.into_std().await).map_err(anyhow::Error::new)?;
        macro_rules! parse {
            (|$name:ident = $file:literal| $block:expr) => {
                let pack = if let Ok($name) = zip.by_name($file) {
                    let data = $block;
                    Some(data)
                } else {
                    None
                };
                // Need separate block for return as a workaround for this: https://github.com/rust-lang/rust/issues/92985
                if let Some(mods) = pack {
                    return Ok(ModpackData { archive: zip, mods });
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
            PackMods::Modrinth { known, unknown }
        });
        parse!(|manifest = "manifest.json"| PackMods::Forge(parse_forge(read_json!(manifest))?));
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

                for f in files.iter() {
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
                    if let Ok((pid, vid)) = f.index_version() {
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
                    } else {
                        pending.push(f);
                        continue;
                    };
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

fn parse_forge(manifest: ModpackManifest) -> Result<HashMap<ProjectId, VersionId>> {
    match manifest {
        ModpackManifest::V1 { files, .. } => Ok(files
            .into_iter()
            .map(|f| (ProjectId::Forge(f.project_id), VersionId::Forge(f.file_id)))
            .collect()),
    }
}
