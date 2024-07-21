#![allow(clippy::cast_sign_loss)]

use std::collections::BTreeSet;

use async_scoped::TokioScope;
use github::models::repos::Asset;

use super::{
    schema::{GameVersion, Mod, Modpack, Project, ProjectId, ProjectIdSvcType, Version, VersionId, VersionIdSvcType},
    ApiOps, GithubClient,
};
use crate::{config::ModLoader, ErrorKind, Result};

impl ApiOps for GithubClient {
    super::get_latest!();

    super::get_version!();

    async fn get_mod(&self, id: &(impl ProjectIdSvcType + ?Sized)) -> Result<Mod> {
        fetch_repo(self, id.get_github()?).await.map(Mod)
    }

    // No distinction between mods and modpacks for github
    async fn get_modpack(&self, id: &(impl ProjectIdSvcType + ?Sized)) -> Result<Modpack> {
        fetch_repo(self, id.get_github()?).await.map(Modpack)
    }

    async fn get_mods(&self, ids: &[&dyn ProjectIdSvcType]) -> Result<Vec<Mod>> {
        // let repos = ids
        //     .iter()
        //     .filter_map(|id| id.get_github().ok())
        //     .format_with(" ", |(own, repo), f| f(&format_args!("repo:{own}/{repo}")))
        //     .to_string();
        // let query = format!(
        //     "query($q:String!){{\
        //         search(type:REPOSITORY,query:$q,first:100){{\
        //             ... on Repository{{\
        //                 owner:owner.login \
        //                 name \
        //                 slug:full_name \
        //                 created:created_at \
        //                 updated:updated_at \
        //                 authors:topics \
        //                 website:url \
        //                 source_url:url \
        //                 license{{\
        //                     name \
        //                     spdx_id \
        //                     url:html_url \
        //                 }\
        //             }\
        //         }\
        //     }\
        //     variables{}",
        //     json!({"q": repos})
        // );
        // let resp = self.graphql(&HashMap::from([("query", query)])).await?;

        // FIXME: Rate limiting
        let ((), mods) = TokioScope::scope_and_block(|s| {
            for id in ids {
                s.spawn(self.get_mod(id));
            }
        });
        let mods = mods.into_iter().filter_map(|r| r.ok().and_then(Result::ok)).collect();
        Ok(mods)
    }

    async fn get_project_versions(
        &self,
        id: &(impl ProjectIdSvcType + ?Sized),
        game_version: Option<&str>,
        loader: Option<ModLoader>,
    ) -> Result<Vec<Version>> {
        async fn _get_project_versions(
            this: &GithubClient,
            (owner, repo): (&str, &str),
            game_version: Option<&str>,
            loader: Option<ModLoader>,
        ) -> Result<Vec<Version>> {
            let filter = [game_version.unwrap_or_default(), loader.map(ModLoader::as_str).unwrap_or_default()];
            let check = |a: &Asset| {
                macro_rules! check_ext {
                    ($ext:literal) => {
                        std::path::Path::new(&a.name)
                            .extension()
                            .map_or(false, |ext| ext.eq_ignore_ascii_case($ext))
                    };
                }
                filter.iter().all(|f| a.name.contains(f))
                    && (check_ext!(".jar") || check_ext!(".zip"))
                    && !a.name.ends_with("-sources.jar")
                    && !a.label.as_ref().is_some_and(|l| l == "Source code")
            };

            let files = this
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
                            project_id: ProjectId::Github((owner.to_owned(), repo.to_owned())),
                            title: a.label.unwrap_or_default(),
                            download_url: Some(a.url),
                            filename: a.name.try_into().expect("Github API should always return a proper relative file"),
                            length: a.size as _,
                            date: a.updated_at.to_rfc3339(),
                            sha1: None,
                            deps: vec![],
                            game_versions: game_version.iter().map(ToString::to_string).collect(),
                            loaders: loader.iter().copied().collect(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Ok(files)
        }
        _get_project_versions(self, id.get_github()?, game_version, loader).await
    }

    async fn get_game_versions(&self) -> Result<BTreeSet<GameVersion>> {
        Err(ErrorKind::Unsupported.into())
    }

    async fn get_versions(&self, _ids: &[&dyn VersionIdSvcType]) -> Result<Vec<Version>> {
        // Rest API doesn't support getting arbitrary assets by id. Needs GraphQL
        Err(ErrorKind::Unsupported.into())
    }
}

#[inline]
async fn fetch_repo(client: &GithubClient, (owner, name): (&str, &str)) -> Result<Project> {
    Ok(client.repos(owner, name).get().await?.into())
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
