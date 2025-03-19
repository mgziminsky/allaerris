#![allow(clippy::cast_sign_loss)]

use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use async_scoped::TokioScope;
use github::models::repos::Asset;

use super::{
    ApiOps, GithubClient, common,
    schema::{GameVersion, Project, ProjectId, ProjectIdSvcType, Version, VersionId, VersionIdSvcType},
};
use crate::{
    ErrorKind, Result,
    config::{ModLoader, VersionedProject},
    mgmt::LockedMod,
};

impl ApiOps for GithubClient {
    common::get_latest!();

    common::get_version!();

    async fn get_project(&self, id: &(impl ProjectIdSvcType + ?Sized)) -> Result<Project> {
        fetch_repo(self, id.get_github()?).await
    }

    async fn get_projects(&self, ids: &[&dyn ProjectIdSvcType]) -> Result<Vec<Project>> {
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

        let ids: Vec<_> = ids.iter().filter_map(|id| id.get_github().ok()).collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }
        // FIXME: Rate limiting
        let ((), mods) = TokioScope::scope_and_block(|s| {
            for id in ids {
                s.spawn(fetch_repo(self, id));
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
        async fn get_project_versions(
            this: &GithubClient,
            (owner, repo): (&str, &str),
            game_version: Option<&str>,
            loader: Option<ModLoader>,
        ) -> Result<Vec<Version>> {
            let filter = [game_version.unwrap_or_default(), loader.map(ModLoader::as_str).unwrap_or_default()];
            let check = |a: &Asset| {
                macro_rules! check_ext {
                    ($($ext:literal)||*) => {
                        std::path::Path::extension(a.name.as_ref()).map_or(false, |ext| $(ext.eq_ignore_ascii_case($ext))||*)
                    };
                }
                filter.iter().all(|f| a.name.contains(f))
                    && check_ext!(".jar" || ".zip" || ".mrpack")
                    && !a.name.ends_with("-sources.jar")
                    && a.label.as_ref().is_none_or(|l| l != "Source code")
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
        get_project_versions(self, id.get_github()?, game_version, loader).await
    }

    async fn get_game_versions(&self) -> Result<BTreeSet<GameVersion>> {
        Err(ErrorKind::Unsupported.into())
    }

    async fn get_versions(&self, ids: &[&dyn VersionIdSvcType]) -> Result<Vec<Version>> {
        let ids: Vec<_> = ids.iter().filter_map(|id| id.get_github().ok()).collect();
        if ids.is_empty() {
            return Ok(vec![]);
        }
        // Rest API doesn't support getting arbitrary assets by id. Needs GraphQL
        Err(ErrorKind::Unsupported.into())
    }

    async fn get_updates(&self, game_version: &str, loader: ModLoader, mods: &[&LockedMod]) -> Result<Vec<LockedMod>> {
        let mods: Vec<_> = mods.iter().filter(|lm| matches!(lm.project(), ProjectId::Github(_))).collect();
        if mods.is_empty() {
            return Ok(vec![]);
        }

        let mut updates = vec![];
        for m in mods {
            if let Ok(up) = self.get_latest(m.project(), Some(game_version), Some(loader)).await {
                if up.id != m.version().unwrap() {
                    updates.push(up.into());
                }
            }
        }

        Ok(updates)
    }

    async fn lookup(&self, _files: &[impl AsRef<Path>], _out_results: &mut HashMap<PathBuf, Version>) -> Result<Vec<crate::Error>> {
        // Use Ok so multi client doesn't fail...
        Ok(vec![ErrorKind::Unsupported.into()])
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
        ErrorKind,
        client::{
            Client, ClientInner, GithubClient,
            schema::{self, Project, ProjectId, ProjectType},
        },
        github::models::Repository,
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
            let slug = repo.full_name.expect("repo should always have full_name");
            Self {
                id: ProjectId::Github((owner.login.clone(), repo.name.clone())),
                project_type: guess_type(&slug),
                name: repo.name,
                slug,
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


    fn guess_type(repo: &str) -> ProjectType {
        let repo = repo.to_ascii_lowercase();
        if repo.contains("modpack") {
            ProjectType::ModPack
        } else if repo.contains("resourcepack") {
            ProjectType::ResourcePack
        } else if repo.contains("datapack") {
            ProjectType::DataPack
        } else if repo.contains("shader") {
            ProjectType::Shader
        } else {
            ProjectType::Mod
        }
    }
}
