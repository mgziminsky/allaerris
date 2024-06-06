use std::fmt::Display;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::client::service_id::svc_id_impl;

svc_id_impl! {
    /// The [client](crate::client) specific project id types
    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub enum ProjectId {
        Forge(usize),
        Modrinth(String),
        Github((String, String)),
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub icon: Option<Url>,
}

impl Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ProjectId::Forge(id) => f.write_fmt(format_args!("{id}")),
            ProjectId::Modrinth(id) => f.write_str(id),
            ProjectId::Github((owner, name)) => f.write_fmt(format_args!("{}/{}", owner.to_lowercase(), name.to_lowercase())),
        }
    }
}

impl<T: AsRef<str>> PartialEq<T> for ProjectId {
    fn eq(&self, other: &T) -> bool {
        let other = other.as_ref();
        match self {
            ProjectId::Forge(id) => other.parse().is_ok_and(|n: usize| *id == n),
            ProjectId::Modrinth(id) => id == other,
            ProjectId::Github((owner, repo)) => other
                .split_once('/')
                .is_some_and(|(owner_other, repo_other)| owner == owner_other && repo == repo_other),
        }
    }
}
impl<T: AsRef<str>> PartialOrd<T> for ProjectId {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        let other = other.as_ref();
        match self {
            ProjectId::Forge(id) => other.parse().ok().and_then(|n: usize| id.partial_cmp(&n)),
            ProjectId::Modrinth(id) => Some(id.as_str().cmp(other)),
            ProjectId::Github((owner, repo)) => other
                .split_once('/')
                .and_then(|other| (owner.as_str(), repo.as_str()).partial_cmp(&other)),
        }
    }
}
