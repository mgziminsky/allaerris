use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use modrinth::{
    apis::{
        projects_api::{GetProjectParams, GetProjectsParams},
        version_files_api::{GetLatestVersionsFromHashesParams, VersionsFromHashesParams},
        versions_api::{GetProjectVersionsParams, GetVersionParams, GetVersionsParams},
    },
    models::{game_version_tag::VersionType, GetLatestVersionsFromHashesBody, HashList, Project as ApiProject},
};

use super::{
    common::{self, compute_lookup_hashes},
    schema::{GameVersion, Project, ProjectIdSvcType, Version, VersionIdSvcType},
    ApiOps, ModrinthClient,
};
use crate::{
    config::{ModLoader, VersionedProject},
    hash::Sha1Async,
    mgmt::LockedMod,
    Result,
};

impl ApiOps for ModrinthClient {
    common::get_latest!();

    async fn get_project(&self, id: &(impl ProjectIdSvcType + ?Sized)) -> Result<Project> {
        fetch_project(self, id.get_modrinth()?).await.map(Into::into)
    }

    async fn get_projects(&self, ids: &[&dyn ProjectIdSvcType]) -> Result<Vec<Project>> {
        let ids: &Vec<_> = &ids.iter().filter_map(|id| id.get_modrinth().ok()).collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let projects = self
            .projects()
            .get_projects(&GetProjectsParams { ids })
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(projects)
    }

    async fn get_project_versions(
        &self,
        id: &(impl ProjectIdSvcType + ?Sized),
        game_version: Option<&str>,
        loader: Option<ModLoader>,
    ) -> Result<Vec<Version>> {
        let mod_id = id.get_modrinth()?;
        let versions = self
            .versions()
            .get_project_versions(&GetProjectVersionsParams {
                mod_id,
                loaders: loader.and_then(ModLoader::known).map(|l| vec![l.as_str()]),
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

    async fn get_versions(&self, ids: &[&dyn VersionIdSvcType]) -> Result<Vec<Version>> {
        let ids: &Vec<_> = &ids.iter().filter_map(|id| id.get_modrinth().ok()).collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let versions = self
            .versions()
            .get_versions(&GetVersionsParams { ids })
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(versions)
    }

    async fn get_version(&self, id: &(impl VersionIdSvcType + ?Sized)) -> Result<Version> {
        let id = id.get_modrinth()?;
        self.versions()
            .get_version(&GetVersionParams { id })
            .await
            .map(Into::into)
            .map_err(Into::into)
    }

    async fn get_updates(&self, game_version: &str, loader: ModLoader, mods: &[&LockedMod]) -> Result<Vec<LockedMod>> {
        use modrinth::models::get_latest_versions_from_hashes_body::Algorithm;

        let mods = mods
            .iter()
            .filter_map(|m| m.project().get_modrinth().map(|_| (m.sha1.as_str(), *m)).ok())
            .collect::<HashMap<_, _>>();
        if mods.is_empty() {
            return Ok(vec![]);
        }

        let updates = self
            .version_files()
            .get_latest_versions_from_hashes(&GetLatestVersionsFromHashesParams {
                get_latest_versions_from_hashes_body: Some(&GetLatestVersionsFromHashesBody {
                    hashes: mods.keys().copied().collect(),
                    algorithm: Algorithm::Sha1,
                    game_versions: vec![game_version],
                    // API bugged and loader filters don't work on modpacks
                    loaders: vec![/* loader.as_str() */],
                }),
            })
            .await?
            .into_iter()
            .filter_map(|(sha1, v)| {
                // Check loader here since api filter doesn't work
                (mods[sha1.as_str()].version().unwrap() != &v.id && v.loaders.iter().any(|l| l == loader.as_str()))
                    .then_some(Version::from(v).into())
            })
            .collect();

        Ok(updates)
    }

    async fn lookup(&self, files: &[impl AsRef<Path>], out_results: &mut HashMap<PathBuf, Version>) -> Result<Vec<crate::Error>> {
        let (hashes, errors) = compute_lookup_hashes(files, out_results, |p| async move {
            use tokio::{fs, io};
            let mut sha1 = Sha1Async::new();
            io::copy(&mut fs::File::open(p).await?, &mut sha1).await?;
            Ok(sha1.finalize_str())
        });
        if hashes.is_empty() {
            return Ok(errors);
        }

        let versions = self
            .version_files()
            .versions_from_hashes(&VersionsFromHashesParams {
                hash_list: Some(&HashList {
                    hashes: hashes.keys().cloned().collect(),
                    algorithm: modrinth::models::hash_list::Algorithm::Sha1,
                }),
            })
            .await?;

        for (sha1, version) in versions {
            if let Some(path) = hashes.get(&sha1) {
                out_results.insert(path.to_path_buf(), version.into());
            }
        }

        Ok(errors)
    }
}

#[inline]
async fn fetch_project(client: &ModrinthClient, mod_id: &str) -> Result<ApiProject> {
    client
        .projects()
        .get_project(&GetProjectParams { mod_id })
        .await
        .map_err(Into::into)
}

mod from {
    use std::{str::FromStr, sync::LazyLock};

    use modrinth::{
        models::{
            project::ProjectType, version_dependency::DependencyType as ModrinthDepType, GameVersionTag, Project as ApiProject,
            ProjectLicense, Version as ApiVersion, VersionDependency,
        },
        Error as ApiError, ErrorResponse,
    };
    use reqwest::StatusCode;
    use url::Url;

    use crate::{
        client::{
            schema::{self, Author, GameVersion, ProjectId, VersionId},
            Client, ClientInner, ModrinthClient,
        },
        config::ModLoader,
        ErrorKind,
    };

    static HOME: LazyLock<Url> = LazyLock::new(|| "https://modrinth.com/".parse().expect("base url should always parse successfully"));

    impl From<ModrinthClient> for Client {
        fn from(value: ModrinthClient) -> Self {
            ClientInner::Modrinth(value).into()
        }
    }

    impl From<ApiError> for ErrorKind {
        fn from(value: ApiError) -> Self {
            match value.kind() {
                modrinth::ErrorKind::Response(ErrorResponse { status, .. }) if *status == StatusCode::NOT_FOUND => Self::DoesNotExist,
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
                    .join(proj_type_path(project.project_type))
                    .and_then(|url| url.join(&project.slug))
                    .ok(),
                slug: project.slug,
                description: project.description,
                project_type: project.project_type.into(),
                created: Some(project.published),
                updated: Some(project.updated),
                icon: project.icon_url,
                downloads: project.downloads.try_into().unwrap_or_default(),
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
                download_url: Some(file.url),
                filename: file
                    .filename
                    .try_into()
                    .expect("Modrinth API should always return a proper relative file"),
                length: file.size.try_into().unwrap_or_default(),
                date: value.date_published,
                sha1: Some(file.hashes.sha1),
                deps: value.dependencies.into_iter().filter_map(|d| d.try_into().ok()).collect(),
                game_versions: value.game_versions,
                loaders: value
                    .loaders
                    .into_iter()
                    .filter_map(|l| ModLoader::from_str(&l).expect("infallible conversion").known())
                    .collect(),
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
                _ => Self::Other,
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

    impl From<ProjectType> for schema::ProjectType {
        fn from(typ: ProjectType) -> Self {
            match typ {
                ProjectType::Mod => Self::Mod,
                ProjectType::Modpack => Self::ModPack,
                ProjectType::Resourcepack => Self::ResourcePack,
                ProjectType::Shader => Self::Shader,
            }
        }
    }

    const fn proj_type_path(ty: ProjectType) -> &'static str {
        // Trailing slash is necessary for Url::join
        match ty {
            ProjectType::Mod => "/mod/",
            ProjectType::Modpack => "/modpack/",
            ProjectType::Resourcepack => "/resourcepack/",
            ProjectType::Shader => "/shader/",
        }
    }
}
