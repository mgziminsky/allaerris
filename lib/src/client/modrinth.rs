use modrinth::{
    apis::{
        projects_api::{GetProjectParams, GetProjectsParams},
        versions_api::GetProjectVersionsParams,
    },
    models::Project as ApiProject,
};

use super::{
    schema::{Mod, Modpack, ProjectIdSvcType, Version},
    ApiOps, RawModrinthClient,
};
use crate::{config::ModLoader, Result};

impl ApiOps for RawModrinthClient {
    async fn get_mod(&self, id: impl AsRef<str>) -> Result<Mod> {
        fetch_project(self, id.as_ref()).await?.try_into()
    }

    async fn get_modpack(&self, id: impl AsRef<str>) -> Result<Modpack> {
        fetch_project(self, id.as_ref()).await?.try_into()
    }

    async fn get_mods(&self, ids: impl AsRef<[&str]>) -> Result<Vec<Mod>> {
        let projects = self
            .projects()
            .get_projects(&GetProjectsParams { ids: ids.as_ref() })
            .await?
            .into_iter()
            .filter_map(|p| p.try_into().ok())
            .collect();

        Ok(projects)
    }

    async fn get_project_versions(&self, id: impl AsRef<ProjectIdSvcType>, game_version: impl AsRef<Option<&str>>, loader: impl AsRef<Option<ModLoader>>) -> Result<Vec<Version>> {
        let mod_id = id.as_ref().as_modrinth()?;
        let versions = self
            .versions()
            .get_project_versions(&GetProjectVersionsParams {
                mod_id,
                loaders: loader.as_ref().map(|l| vec![l.as_str()]),
                game_versions: game_version.as_ref().map(|v| vec![v]),
                featured: None,
            })
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(versions)
    }
}

async fn fetch_project(client: &RawModrinthClient, mod_id: &str) -> Result<ApiProject> {
    client.projects().get_project(&GetProjectParams { mod_id }).await.map_err(Into::into)
}

mod from {
    use modrinth::{
        models::{project::ProjectType, version_dependency::DependencyType as ModrinthDepType, Project as ApiProject, Version as ApiVersion, VersionDependency},
        Error as ApiError, ErrorResponse,
    };
    use reqwest::StatusCode;

    use crate::{
        client::{
            schema::{self, ProjectId, VersionId},
            Client, ClientInner, RawModrinthClient,
        },
        Error,
    };

    impl From<RawModrinthClient> for Client {
        fn from(value: RawModrinthClient) -> Self {
            ClientInner::Modrinth(value).into()
        }
    }

    impl From<ApiError> for Error {
        fn from(value: ApiError) -> Self {
            match value {
                ApiError::Response(ErrorResponse { status, .. }) if status == StatusCode::NOT_FOUND => Self::DoesNotExist,
                _ => Self::Modrinth(value),
            }
        }
    }

    macro_rules! try_from_project {
        ($($ty:ident),*$(,)?) => {$(
            impl TryFrom<ApiProject> for schema::$ty {
                type Error = crate::Error;
                fn try_from(project: ApiProject) -> Result<Self, Self::Error> {
                    if let ProjectType::$ty = project.project_type {
                        Ok(Self(schema::Project {
                            id: ProjectId::Modrinth(project.id.clone()),
                            name: project.title.clone(),
                            slug: project.slug.clone(),
                            description: project.description,
                            created: Some(project.published),
                            updated: Some(project.updated),
                            icon: project.icon_url,
                        }))
                    } else {
                        Err(Self::Error::WrongType(stringify!($ty)))
                    }
                }
            }
        )*};
    }
    try_from_project! {
        Mod,
        Modpack,
    }

    impl From<ApiVersion> for schema::Version {
        fn from(value: ApiVersion) -> Self {
            let file = {
                let len = value.files.len();
                let mut files = value.files.into_iter();
                if len > 1 {
                    files.find(|f| f.primary)
                } else {
                    files.next()
                }
            }
            .expect("Modrinth version should always have at least 1 file");

            Self {
                id: VersionId::Modrinth(value.id),
                project_id: ProjectId::Modrinth(value.project_id),
                title: value.name,
                download_url: file.url,
                filename: file.filename,
                length: file.size as usize,
                date: value.date_published,
                sha1: Some(file.hashes.sha1),
                deps: value.dependencies.into_iter().filter_map(|d| d.try_into().ok()).collect(),
            }
        }
    }

    impl TryFrom<VersionDependency> for schema::Dependency {
        type Error = ();

        fn try_from(value: VersionDependency) -> Result<Self, Self::Error> {
            if let Some(mod_id) = value.project_id {
                Ok(Self {
                    project_id: ProjectId::Modrinth(mod_id),
                    id: value.version_id.map(VersionId::Modrinth),
                    dep_type: value.dependency_type.into(),
                })
            } else {
                Err(())
            }
        }
    }

    impl From<ModrinthDepType> for schema::DependencyType {
        fn from(value: ModrinthDepType) -> Self {
            match value {
                ModrinthDepType::Required => Self::Required,
                ModrinthDepType::Optional => Self::Optional,
                _ => Self::Optional,
            }
        }
    }
}
