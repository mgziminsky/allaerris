use curseforge::{
    apis::{
        files_api::GetModFilesParams,
        mods_api::{GetModParams, GetModsParams},
    },
    models::GetModsByIdsListRequestBody,
};

use super::{
    schema::{AsProjectId, Mod, Modpack, ProjectIdSvcType, Version},
    ApiOps, ForgeClient,
};
use crate::{config::ModLoader, Result};

impl ApiOps for ForgeClient {
    async fn get_mod(&self, id: impl AsProjectId) -> Result<Mod> {
        fetch_mod(self, id).await?.try_into()
    }

    async fn get_modpack(&self, id: impl AsProjectId) -> Result<Modpack> {
        fetch_mod(self, id).await?.try_into()
    }

    async fn get_mods<T: AsProjectId>(&self, ids: impl AsRef<[T]>) -> Result<Vec<Mod>> {
        let mod_ids = ids.as_ref().into_iter().filter_map(|i| i.try_as_forge().ok()).collect();
        let mods = self
            .mods()
            .get_mods(&GetModsParams {
                get_mods_by_ids_list_request_body: &GetModsByIdsListRequestBody { mod_ids },
            })
            .await?
            .data
            .into_iter()
            .filter_map(|m| m.try_into().ok())
            .collect();

        Ok(mods)
    }

    async fn get_project_versions(
        &self,
        id: impl AsRef<ProjectIdSvcType>,
        game_version: impl AsRef<Option<&str>>,
        loader: impl AsRef<Option<ModLoader>>,
    ) -> Result<Vec<Version>> {
        let id = id.as_ref().as_forge()?;
        let files = self
            .files()
            .get_mod_files(&GetModFilesParams {
                mod_id: *id as _,
                game_version: *game_version.as_ref(),
                mod_loader_type: loader.as_ref().map(Into::into),
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
}

#[inline]
async fn fetch_mod(client: &ForgeClient, id: impl AsProjectId) -> Result<curseforge::models::Mod> {
    let mod_id = id.try_as_forge()? as _;
    Ok(client.mods().get_mod(&GetModParams { mod_id }).await?.data)
}

mod from {
    use curseforge::{
        models::{File, FileDependency, FileRelationType, HashAlgo, ModAuthor, ModLoaderType},
        Error as ApiError, ErrorResponse,
    };
    use once_cell::sync::Lazy;
    use reqwest::StatusCode;
    use url::Url;

    use crate::{
        client::{
            schema::{Author, Dependency, DependencyType, Mod, Modpack, Project, ProjectId, Version, VersionId},
            Client, ClientInner, ForgeClient,
        },
        config::ModLoader,
        ErrorKind,
    };

    const MINECRAFT_GAME_ID: u64 = 432;
    const MOD_CLASS_ID: u32 = 6;
    const MODPACK_CLASS_ID: u32 = 4471;

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
                website: value
                    .class_id
                    .and_then(class_slug)
                    .and_then(|class| HOME.join(class).ok())
                    .and_then(|url| url.join(&value.slug).ok()),
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
        ($($ty:ty = $val:ident),* $(,)?) => {$(
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

    impl From<File> for Version {
        fn from(file: File) -> Self {
            Self {
                id: VersionId::Forge(file.id as _),
                project_id: ProjectId::Forge(file.mod_id as _),
                title: file.display_name,
                download_url: file.download_url,
                filename: file.file_name,
                length: file.file_length as _,
                date: file.file_date,
                sha1: file.hashes.into_iter().find(|h| matches!(h.algo, HashAlgo::Sha1)).map(|h| h.value),
                deps: file.dependencies.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<FileDependency> for Dependency {
        fn from(value: FileDependency) -> Self {
            Self {
                project_id: ProjectId::Forge(value.mod_id as _),
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


    /// Values taken from the api as of 2024-06
    fn class_slug(cid: u32) -> Option<&'static str> {
        Some(match cid {
            5 => "bukkit-plugins",
            6 => "mc-mods",
            12 => "texture-packs",
            17 => "worlds",
            4471 => "modpacks",
            4546 => "customization",
            4559 => "mc-addons",
            6552 => "shaders",
            6945 => "data-packs",
            _ => return None,
        })
    }
}
