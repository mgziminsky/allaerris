use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use curseforge::{
    apis::{
        files_api::{GetFilesParams, GetModFilesParams},
        fingerprints_api::GetFingerprintMatchesByGameParams,
        minecraft_api::GetMinecraftVersionsParams,
        mods_api::{GetModParams, GetModsParams},
    },
    models::{GetFingerprintMatchesRequestBody, GetModFilesRequestBody, GetModsByIdsListRequestBody},
};

use super::{
    common::{self, compute_lookup_hashes},
    schema::{GameVersion, Mod, Modpack, ProjectIdSvcType, Version, VersionIdSvcType},
    ApiOps, ForgeClient,
};
use crate::{
    client::schema::{ProjectId, VersionId},
    config::{ModLoader, ProjectWithVersion, VersionedProject},
    hash,
    mgmt::LockedMod,
    Result,
};

impl ApiOps for ForgeClient {
    common::get_latest!();

    common::get_version!();

    async fn get_mod(&self, id: &(impl ProjectIdSvcType + ?Sized)) -> Result<Mod> {
        fetch_mod(self, id).await?.try_into()
    }

    async fn get_modpack(&self, id: &(impl ProjectIdSvcType + ?Sized)) -> Result<Modpack> {
        fetch_mod(self, id).await?.try_into()
    }

    async fn get_mods(&self, ids: &[&dyn ProjectIdSvcType]) -> Result<Vec<Mod>> {
        let ids: Vec<_> = ids.iter().filter_map(|i| i.get_forge().ok()).collect();
        let mods = fetch_mods(self, ids).await?.into_iter().filter_map(|m| m.try_into().ok()).collect();
        Ok(mods)
    }

    async fn get_project_versions(
        &self,
        id: &(impl ProjectIdSvcType + ?Sized),
        game_version: Option<&str>,
        loader: Option<ModLoader>,
    ) -> Result<Vec<Version>> {
        let mod_id = id.get_forge()?;
        let files = self
            .files()
            .get_mod_files(&GetModFilesParams {
                mod_id,
                game_version,
                mod_loader_type: loader.map(Into::into),
                game_version_type_id: None,
                index: None,
                page_size: None,
            })
            .await?
            .data
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(files)
    }

    async fn get_game_versions(&self) -> Result<BTreeSet<GameVersion>> {
        Ok(self
            .minecraft()
            .get_minecraft_versions(&GetMinecraftVersionsParams { sort_descending: None })
            .await?
            .data
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn get_versions(&self, ids: &[&dyn VersionIdSvcType]) -> Result<Vec<Version>> {
        let file_ids: Vec<_> = ids.iter().filter_map(|i| i.get_forge().ok()).collect();
        if file_ids.is_empty() {
            return Ok(vec![]);
        }
        let versions = self
            .files()
            .get_files(&GetFilesParams {
                get_mod_files_request_body: &GetModFilesRequestBody { file_ids },
            })
            .await?
            .data
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(versions)
    }

    async fn get_updates(&self, game_version: &str, loader: ModLoader, mods: &[&LockedMod]) -> Result<Vec<LockedMod>> {
        let mods = &mods
            .iter()
            .filter_map(|m| m.project().get_forge().map(|id| (id, *m)).ok())
            .collect::<HashMap<_, _>>();
        let data = fetch_mods(self, mods.keys().copied().collect()).await?;
        if data.is_empty() {
            return Ok(vec![]);
        }

        let updates = data
            .into_iter()
            .flat_map(|m| {
                m.latest_file_indexes.into_iter().filter_map(move |fi| {
                    let vid = mods[&m.id].version().unwrap();
                    if fi.mod_loader == loader.into() && vid != &fi.file_id && fi.game_version == game_version {
                        fi.filename.try_into().ok().map(|file| LockedMod {
                            id: ProjectWithVersion::new(ProjectId::Forge(m.id), Some(VersionId::Forge(fi.file_id)))
                                .unwrap()
                                .try_into()
                                .unwrap(),
                            sha1: String::new(),
                            file,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(updates)
    }

    async fn lookup(&self, files: &[impl AsRef<Path>], out_results: &mut HashMap<PathBuf, Version>) -> Result<Vec<crate::Error>> {
        let (fprints, errors) = compute_lookup_hashes(files, out_results, hash::forge_fingerprint);
        if fprints.is_empty() {
            return Ok(errors);
        }

        let versions = self
            .fingerprints()
            .get_fingerprint_matches_by_game(&GetFingerprintMatchesByGameParams {
                game_id: from::MINECRAFT_GAME_ID,
                get_fingerprint_matches_request_body: &GetFingerprintMatchesRequestBody {
                    fingerprints: fprints.keys().copied().collect(),
                },
            })
            .await?
            .data
            .exact_matches;

        for v in versions {
            if let Some(path) = fprints.get(&v.file.file_fingerprint) {
                out_results.insert(path.to_path_buf(), v.file.into());
            }
        }

        Ok(errors)
    }
}

async fn fetch_mod(client: &ForgeClient, id: impl ProjectIdSvcType) -> Result<curseforge::models::Mod> {
    let mod_id = id.get_forge()?;
    Ok(client.mods().get_mod(&GetModParams { mod_id }).await?.data)
}

async fn fetch_mods(client: &ForgeClient, mod_ids: Vec<u64>) -> Result<Vec<curseforge::models::Mod>> {
    if mod_ids.is_empty() {
        return Ok(vec![]);
    }
    let data = client
        .mods()
        .get_mods(&GetModsParams {
            get_mods_by_ids_list_request_body: &GetModsByIdsListRequestBody { mod_ids },
        })
        .await?
        .data;
    Ok(data)
}

mod from {
    use curseforge::{
        models::{File, FileDependency, FileRelationType, HashAlgo, MinecraftGameVersion, ModAuthor, ModLoaderType},
        Error as ApiError, ErrorResponse,
    };
    use once_cell::sync::Lazy;
    use reqwest::StatusCode;
    use url::Url;

    use crate::{
        client::{
            schema::{Author, Dependency, DependencyType, GameVersion, Mod, Modpack, Project, ProjectId, Version, VersionId},
            Client, ClientInner, ForgeClient,
        },
        config::ModLoader,
        ErrorKind,
    };

    pub const MINECRAFT_GAME_ID: u64 = 432;
    pub const MOD_CLASS_ID: u64 = 6;
    pub const MODPACK_CLASS_ID: u64 = 4471;

    static HOME: Lazy<Url> = Lazy::new(|| {
        "https://www.curseforge.com/minecraft/"
            .parse()
            .expect("base url should always parse successfully")
    });


    impl From<ForgeClient> for Client {
        fn from(value: ForgeClient) -> Self {
            ClientInner::Forge(value).into()
        }
    }

    impl From<ApiError> for ErrorKind {
        fn from(value: ApiError) -> Self {
            match value {
                ApiError::Response(ErrorResponse { status, .. }) if status == StatusCode::NOT_FOUND => Self::DoesNotExist,
                _ => Self::Forge(value),
            }
        }
    }

    impl From<curseforge::models::Mod> for Project {
        fn from(value: curseforge::models::Mod) -> Self {
            Self {
                id: ProjectId::Forge(value.id),
                name: value.name,
                website: proj_website(value.class_id, &value.slug),
                slug: value.slug,
                description: value.summary,
                created: Some(value.date_released),
                updated: Some(value.date_modified),
                icon: Some(value.logo.thumbnail_url),
                authors: value.authors.into_iter().map(Into::into).collect(),
                categories: value.categories.into_iter().map(|c| c.name).collect(),
                license: None,
                downloads: value.download_count,
                source_url: Some(value.links.source_url),
            }
        }
    }

    macro_rules! try_from {
        ($($ty:ty = $val:path),* $(,)?) => {$(
            impl TryFrom<curseforge::models::Mod> for $ty {
                type Error = crate::Error;
                fn try_from(value: curseforge::models::Mod) -> Result<Self, Self::Error> {
                    if value.game_id != MINECRAFT_GAME_ID {
                        return Err(ErrorKind::Incompatible)?;
                    }

                    if value.class_id != Some($val) {
                        return Err(ErrorKind::WrongType(stringify!($ty)))?;
                    }

                    Ok(Self(value.into()))
                }
            }
        )*};
    }
    try_from! {
        Mod = MOD_CLASS_ID,
        Modpack = MODPACK_CLASS_ID,
    }

    impl From<ModLoader> for ModLoaderType {
        fn from(value: ModLoader) -> Self {
            match value {
                ModLoader::Unknown => Self::Any,
                ModLoader::Forge => Self::Forge,
                ModLoader::Cauldron => Self::Cauldron,
                ModLoader::LiteLoader => Self::LiteLoader,
                ModLoader::Fabric => Self::Fabric,
                ModLoader::Quilt => Self::Quilt,
                ModLoader::NeoForge => Self::NeoForge,
            }
        }
    }

    const VERSION_TYPE_ID: u64 = 75125;
    const LOADER_TYPE_ID: u64 = 68441;
    impl From<File> for Version {
        fn from(file: File) -> Self {
            let (game_versions, loaders) = file.sortable_game_versions.into_iter().fold((vec![], vec![]), |mut acc, version| {
                let (gv, l) = &mut acc;
                match version.game_version_type_id {
                    Some(VERSION_TYPE_ID) => gv.push(version.game_version),
                    Some(LOADER_TYPE_ID) => match version.game_version_name.parse() {
                        Ok(ModLoader::Unknown) => { /* Skip Unknown Loaders */ },
                        Ok(loader) => l.push(loader),
                        Err(_) => unreachable!(),
                    },
                    _ => { /* Skip other types */ },
                }
                acc
            });
            Self {
                id: VersionId::Forge(file.id),
                project_id: ProjectId::Forge(file.mod_id),
                title: file.display_name,
                download_url: file.download_url,
                filename: file
                    .file_name
                    .try_into()
                    .expect("Curseforge API should always return a proper relative file"),
                length: file.file_length as _,
                date: file.file_date,
                sha1: file.hashes.into_iter().find(|h| matches!(h.algo, HashAlgo::Sha1)).map(|h| h.value),
                deps: file.dependencies.into_iter().map(Into::into).collect(),
                game_versions,
                loaders,
            }
        }
    }

    impl From<FileDependency> for Dependency {
        fn from(value: FileDependency) -> Self {
            Self {
                project_id: ProjectId::Forge(value.mod_id),
                id: None,
                dep_type: value.relation_type.into(),
            }
        }
    }

    impl From<FileRelationType> for DependencyType {
        fn from(value: FileRelationType) -> Self {
            match value {
                FileRelationType::RequiredDependency => Self::Required,
                FileRelationType::OptionalDependency => Self::Optional,
                _ => Self::Other,
            }
        }
    }

    impl From<ModAuthor> for Author {
        fn from(ModAuthor { name, url, .. }: ModAuthor) -> Self {
            Self { name, url: Some(url) }
        }
    }

    impl From<MinecraftGameVersion> for GameVersion {
        fn from(gv: MinecraftGameVersion) -> Self {
            Self {
                version: gv.version_string,
                release_date: gv.date_modified,
            }
        }
    }


    fn proj_website(class_id: Option<u64>, slug: &str) -> Option<Url> {
        class_id
            .and_then(class_slug)
            .and_then(|class| HOME.join(class).ok())
            .and_then(|url| url.join(slug).ok())
    }

    /// Values taken from the api as of 2024-06
    const fn class_slug(cid: u64) -> Option<&'static str> {
        // NOTE: No leading slash, and trailing slash are required for url joining to
        // work properly
        Some(match cid {
            5 => "bukkit-plugins/",
            6 => "mc-mods/",
            12 => "texture-packs/",
            17 => "worlds/",
            4471 => "modpacks/",
            4546 => "customization/",
            4559 => "mc-addons/",
            6552 => "shaders/",
            6945 => "data-packs/",
            _ => return None,
        })
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn proj_website_none() {
            assert_eq!(proj_website(None, "test"), None);
        }

        #[test]
        fn proj_website_mod() {
            const EXPECTED: &str = "/mc-mods/test";

            let site = proj_website(Some(MOD_CLASS_ID), "test").expect("should produce a valid url");
            let path = site.path();
            assert_eq!(&path[path.len() - EXPECTED.len()..], EXPECTED);
        }

        #[test]
        fn proj_website_pack() {
            const EXPECTED: &str = "/modpacks/test";

            let site = proj_website(Some(MODPACK_CLASS_ID), "test").expect("should produce a valid url");
            let path = site.path();
            assert_eq!(&path[path.len() - EXPECTED.len()..], EXPECTED);
        }
    }
}
