use std::collections::BTreeSet;

use modrinth::{
    apis::{
        projects_api::{GetProjectParams, GetProjectsParams},
        versions_api::GetProjectVersionsParams,
    },
    models::{game_version_tag::VersionType, Project as ApiProject},
};

use super::{
    schema::{AsProjectId, GameVersion, Mod, Modpack, ProjectIdSvcType, Version},
    ApiOps, ModrinthClient,
};
use crate::{config::ModLoader, Result};

impl ApiOps for ModrinthClient {
    async fn get_mod(&self, id: &impl AsProjectId) -> Result<Mod> {
        fetch_project(self, id).await?.try_into()
    }

    async fn get_modpack(&self, id: &impl AsProjectId) -> Result<Modpack> {
        fetch_project(self, id).await?.try_into()
    }

    async fn get_mods(&self, ids: &[impl AsProjectId]) -> Result<Vec<Mod>> {
        let ids: Vec<_> = ids.iter().filter_map(|id| id.try_as_modrinth().ok()).collect();
        let projects = self
            .projects()
            .get_projects(&GetProjectsParams { ids: &ids })
            .await?
            .into_iter()
            .filter_map(|p| p.try_into().ok())
            .collect();

        Ok(projects)
    }

    async fn get_project_versions(
        &self,
        id: &ProjectIdSvcType,
        game_version: Option<&str>,
        loader: Option<ModLoader>,
    ) -> Result<Vec<Version>> {
        let mod_id = id.as_modrinth()?;
        let versions = self
            .versions()
            .get_project_versions(&GetProjectVersionsParams {
                mod_id,
                loaders: loader.map(|l| vec![l.as_str()]),
                game_versions: game_version.map(|v| vec![v]),
                featured: None,
            })
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(versions)
    }

    async fn get_game_versions(&self) -> Result<BTreeSet<GameVersion>> {
        Ok(self
            .tags()
            .version_list()
            .await?
            .into_iter()
            // Only keep full releases
            .filter(|v| v.version_type == VersionType::Release)
            .map(Into::into)
            .collect())
    }
}

#[inline]
async fn fetch_project(client: &ModrinthClient, id: impl AsProjectId) -> Result<ApiProject> {
    let mod_id = id.try_as_modrinth()?;
    client
        .projects()
        .get_project(&GetProjectParams { mod_id })
        .await
        .map_err(Into::into)
}

mod from {
    use modrinth::{
        models::{
            project::ProjectType, version_dependency::DependencyType as ModrinthDepType, GameVersionTag, Project as ApiProject,
            ProjectLicense, Version as ApiVersion, VersionDependency,
        },
        Error as ApiError, ErrorResponse,
    };
    use once_cell::sync::Lazy;
    use reqwest::StatusCode;
    use url::Url;

    use crate::{
        client::{
            schema::{self, Author, GameVersion, ProjectId, VersionId},
            Client, ClientInner, ModrinthClient,
        },
        ErrorKind,
    };

    static HOME: Lazy<Url> = Lazy::new(|| "https://modrinth.com/".parse().expect("base url should always parse successfully"));

    impl From<ModrinthClient> for Client {
        fn from(value: ModrinthClient) -> Self {
            ClientInner::Modrinth(value).into()
        }
    }

    impl From<ApiError> for ErrorKind {
        fn from(value: ApiError) -> Self {
            match value {
                ApiError::Response(ErrorResponse { status, .. }) if status == StatusCode::NOT_FOUND => Self::DoesNotExist,
                _ => Self::Modrinth(value),
            }
        }
    }

    impl From<ApiProject> for schema::Project {
        fn from(project: ApiProject) -> Self {
            Self {
                id: ProjectId::Modrinth(project.id),
                name: project.title,
                website: HOME
                    .join(ProjTypeSlug(project.project_type).as_str())
                    .and_then(|url| url.join(&project.slug))
                    .ok(),
                slug: project.slug,
                description: project.description,
                created: Some(project.published),
                updated: Some(project.updated),
                icon: project.icon_url,
                downloads: project.downloads as _,
                authors: vec![Author {
                    name: project.team,
                    url: None,
                }],
                categories: project.categories,
                license: Some(project.license.into()),
                source_url: project.source_url,
            }
        }
    }

    macro_rules! try_from {
        ($($ty:ident),*$(,)?) => {$(
            impl TryFrom<ApiProject> for schema::$ty {
                type Error = crate::Error;
                fn try_from(project: ApiProject) -> Result<Self, Self::Error> {
                    if let ProjectType::$ty = project.project_type {
                        Ok(Self(project.into()))
                    } else {
                        Err(ErrorKind::WrongType(stringify!($ty)))?
                    }
                }
            }
        )*};
    }
    try_from! {
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

    impl From<ProjectLicense> for schema::License {
        fn from(ProjectLicense { id, name, url }: ProjectLicense) -> Self {
            Self { name, spdx_id: id, url }
        }
    }

    impl From<GameVersionTag> for GameVersion {
        fn from(gv: GameVersionTag) -> Self {
            Self {
                version: gv.version,
                release_date: gv.date,
            }
        }
    }


    struct ProjTypeSlug(ProjectType);
    impl ProjTypeSlug {
        pub fn as_str(&self) -> &str {
            match self.0 {
                ProjectType::Mod => "mod",
                ProjectType::Modpack => "modpack",
                ProjectType::Resourcepack => "resourcepack",
                ProjectType::Shader => "shader",
            }
        }
    }
}
