use curseforge::{
    apis::{
        files_api::GetModFilesParams,
        mods_api::{GetModParams, GetModsParams},
    },
    models::GetModsByIdsListRequestBody,
};

use super::{
    schema::{Mod, Modpack, ProjectIdSvcType, Version},
    ApiOps, RawForgeClient,
};
use crate::{config::ModLoader, Error, Result};

impl ApiOps for RawForgeClient {
    async fn get_mod(&self, id: impl AsRef<str>) -> Result<Mod> {
        fetch_mod(self, id.as_ref()).await?.try_into()
    }

    async fn get_modpack(&self, id: impl AsRef<str>) -> Result<Modpack> {
        fetch_mod(self, id.as_ref()).await?.try_into()
    }

    async fn get_mods(&self, ids: impl AsRef<[&str]>) -> Result<Vec<Mod>> {
        let mod_ids = ids.as_ref().into_iter().filter_map(|i| i.parse().ok()).collect();
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

    async fn get_project_versions(&self, id: impl AsRef<ProjectIdSvcType>, game_version: impl AsRef<Option<&str>>, loader: impl AsRef<Option<ModLoader>>) -> Result<Vec<Version>> {
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

async fn fetch_mod(client: &RawForgeClient, id: &str) -> Result<curseforge::models::Mod> {
    let mod_id = id.parse().or(Err(Error::InvalidIdentifier))?;
    Ok(client.mods().get_mod(&GetModParams { mod_id }).await?.data)
}

mod from {
    use curseforge::{
        models::{File, FileDependency, FileRelationType, HashAlgo, ModLoaderType},
        Error as ApiError, ErrorResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        client::{
            schema::{Dependency, DependencyType, Mod, Modpack, Project, ProjectId, Version, VersionId},
            Client, ClientInner, RawForgeClient,
        },
        config::ModLoader,
        Error,
    };

    const MINECRAFT_GAME_ID: usize = 432;
    const MOD_CLASS_ID: u32 = 6;
    const MODPACK_CLASS_ID: u32 = 4471;

    impl From<RawForgeClient> for Client {
        fn from(value: RawForgeClient) -> Self {
            ClientInner::Forge(value).into()
        }
    }

    impl From<ApiError> for Error {
        fn from(value: ApiError) -> Self {
            match value {
                ApiError::Response(ErrorResponse { status, .. }) if status == StatusCode::NOT_FOUND => Self::DoesNotExist,
                _ => Self::Forge(value),
            }
        }
    }

    macro_rules! project_try_from {
        ($($ty:ty = $val:ident),* $(,)?) => {$(
            impl TryFrom<curseforge::models::Mod> for $ty {
                type Error = crate::Error;
                fn try_from(value: curseforge::models::Mod) -> Result<Self, Self::Error> {
                    if value.game_id != MINECRAFT_GAME_ID {
                        return Err(Self::Error::Incompatible);
                    }

                    if value.class_id != Some($val) {
                        return Err(Self::Error::WrongType(stringify!($ty)));
                    }

                    Ok(Self(Project {
                        id: ProjectId::Forge(value.id),
                        name: value.name,
                        slug: value.slug,
                        description: value.summary,
                        created: Some(value.date_released),
                        updated: Some(value.date_modified),
                        icon: Some(value.logo.thumbnail_url),
                    }))
                }
            }
        )*};
    }
    project_try_from! {
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
                length: file.file_length,
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
}
