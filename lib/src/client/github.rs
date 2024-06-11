use std::collections::BTreeSet;

use async_scoped::TokioScope;
use github::models::repos::Asset;

use super::{
    schema::{AsProjectId, GameVersion, Mod, Modpack, Project, ProjectId, ProjectIdSvcType, Version, VersionId},
    ApiOps, GithubClient,
};
use crate::{config::ModLoader, ErrorKind, Result};

impl ApiOps for GithubClient {
    async fn get_mod(&self, id: &impl AsProjectId) -> Result<Mod> {
        fetch_repo(self, id).await.map(Mod)
    }

    // No distinction between mods and modpacks for github
    async fn get_modpack(&self, id: &impl AsProjectId) -> Result<Modpack> {
        fetch_repo(self, id).await.map(Modpack)
    }

    async fn get_mods(&self, ids: &[impl AsProjectId]) -> Result<Vec<Mod>> {
        // FIXME: Rate limiting
        let (_, mods) = TokioScope::scope_and_block(|s| {
            for id in ids {
                s.spawn(self.get_mod(id));
            }
        });
        let mods = mods.into_iter().filter_map(|r| r.ok().and_then(|r| r.ok())).collect();
        Ok(mods)
    }

    async fn get_project_versions(
        &self,
        id: &ProjectIdSvcType,
        game_version: Option<&str>,
        loader: Option<ModLoader>,
    ) -> Result<Vec<Version>> {
        let id = id.as_github()?;
        let (owner, repo) = id;

        let filter = [game_version.unwrap_or_default(), loader.map(ModLoader::as_str).unwrap_or_default()];
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

    async fn get_game_versions(&self) -> Result<BTreeSet<GameVersion>> {
        Err(ErrorKind::Unsupported.into())
    }
}

#[inline]
async fn fetch_repo(client: &GithubClient, id: impl AsProjectId) -> Result<Project> {
    let (owner, name) = id.try_as_github()?;
    let repo = client.repos(owner, name).get().await?;
    Ok(repo.into())
}


mod from {
    use github::models::{Author, License};
    use reqwest::StatusCode;

    use crate::{
        client::{
            schema::{self, Project, ProjectId},
            Client, ClientInner, GithubClient,
        },
        github::models::Repository,
        ErrorKind,
    };

    impl From<GithubClient> for Client {
        fn from(value: GithubClient) -> Self {
            ClientInner::Github(value).into()
        }
    }

    impl From<github::Error> for ErrorKind {
        fn from(err: github::Error) -> Self {
            match &err {
                github::Error::GitHub { source, .. } if source.status_code == StatusCode::NOT_FOUND => Self::DoesNotExist,
                _ => Self::GitHub(err),
            }
        }
    }

    impl From<Repository> for Project {
        fn from(repo: Repository) -> Self {
            let owner = repo.owner.expect("repo should always have owner");
            Self {
                id: ProjectId::Github((owner.login.clone(), repo.name.clone())),
                name: repo.name,
                slug: repo.full_name.expect("repo should always have full_name"),
                description: repo.description.unwrap_or_default(),
                created: repo.created_at.map(|d| d.to_rfc3339()),
                updated: repo.updated_at.map(|d| d.to_rfc3339()),
                icon: None,
                downloads: 0,
                authors: vec![owner.into()],
                categories: repo.topics.unwrap_or_default(),
                license: repo.license.map(Into::into),
                website: Some(repo.url.clone()),
                source_url: Some(repo.url),
            }
        }
    }

    impl From<Author> for schema::Author {
        fn from(Author { login, html_url, .. }: Author) -> Self {
            Self {
                name: login,
                url: Some(html_url),
            }
        }
    }

    impl From<License> for schema::License {
        fn from(
            License {
                name, spdx_id, html_url, ..
            }: License,
        ) -> Self {
            Self {
                name,
                spdx_id,
                url: html_url,
            }
        }
    }
}
