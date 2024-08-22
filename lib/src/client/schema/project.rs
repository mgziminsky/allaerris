use url::Url;

use crate::client::svc_id_impl;

svc_id_impl! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
