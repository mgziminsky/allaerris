use github::models::repos::Asset;

use super::{
    schema::{Mod, Modpack, Project, ProjectId, ProjectIdSvcType, Version, VersionId},
    ApiOps, GithubClient,
};
use crate::{config::ModLoader, Error, Result};

impl ApiOps for GithubClient {
    async fn get_mod(&self, id: impl AsRef<str>) -> Result<Mod> {
        fetch_repo(self, id).await.map(Mod)
    }

    // No distinction between mods and modpacks for github
    async fn get_modpack(&self, id: impl AsRef<str>) -> Result<Modpack> {
        fetch_repo(self, id).await.map(Modpack)
    }

    async fn get_mods(&self, ids: impl AsRef<[&str]>) -> Result<Vec<Mod>> {
        let ids = ids.as_ref();
        let mut mods = Vec::with_capacity(ids.len());
        for id in ids {
            if let Ok(m) = self.get_mod(id).await {
                mods.push(m);
            }
        }
        Ok(mods)
    }

    async fn get_project_versions(
        &self,
        id: impl AsRef<ProjectIdSvcType>,
        game_version: impl AsRef<Option<&str>>,
        loader: impl AsRef<Option<ModLoader>>,
    ) -> Result<Vec<Version>> {
        let id = id.as_ref().as_github()?;
        let (owner, repo) = id;

        let filter = [
            game_version.as_ref().unwrap_or_default(),
            loader.as_ref().as_ref().map(ModLoader::as_str).unwrap_or_default(),
        ];
        let check = |a: &Asset| {
            filter.iter().all(|f| a.name.contains(f))
                && (a.name.ends_with(".jar") || a.name.ends_with(".zip"))
                && !a.name.ends_with("-sources.jar")
                && !a.label.as_ref().is_some_and(|l| l == "Source code")
        };

        let files = self
            .repos(owner, repo)
            .releases()
            .list()
            .send()
            .await?
            .items
            .into_iter()
            .flat_map(|r| r.assets)
            .filter_map(|a| {
                if check(&a) {
                    Some(Version {
                        id: VersionId::Github(a.id),
                        project_id: ProjectId::Github(id.clone()),
                        title: a.label.unwrap_or_default(),
                        download_url: a.url,
                        filename: a.name,
                        length: a.size as _,
                        date: a.updated_at.to_rfc3339(),
                        sha1: None,
                        deps: vec![],
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }
}

async fn fetch_repo(client: &GithubClient, id: impl AsRef<str>) -> Result<Project> {
    let (owner, name) = id.as_ref().split_once('/').ok_or(Error::InvalidIdentifier)?;
    let repo = client.repos(owner, name).get().await?;
    Ok(repo.into())
}


mod from {
    use reqwest::StatusCode;

    use crate::{
        client::{
            schema::{Project, ProjectId},
            Client, ClientInner, GithubClient,
        },
        github::models::Repository,
        Error,
    };

    impl From<GithubClient> for Client {
        fn from(value: GithubClient) -> Self {
            ClientInner::Github(value).into()
        }
    }

    impl From<github::Error> for Error {
        fn from(err: github::Error) -> Self {
            match &err {
                github::Error::GitHub { source, .. } if source.status_code == StatusCode::NOT_FOUND => Self::DoesNotExist,
                _ => Self::GitHub(err),
            }
        }
    }

    impl From<Repository> for Project {
        fn from(repo: Repository) -> Self {
            Self {
                id: ProjectId::Github((repo.owner.expect("repo should always have owner").login, repo.name.clone())),
                name: repo.name,
                slug: repo.full_name.expect("repo should always have full_name"),
                description: repo.description.unwrap_or_default(),
                created: repo.created_at.map(|d| d.to_rfc3339()),
                updated: repo.updated_at.map(|d| d.to_rfc3339()),
                icon: None,
            }
        }
    }
}
