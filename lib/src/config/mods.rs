use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModId {
    Forge(usize),
    Modrinth(String),
    GitHub((String, String)),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    #[serde(flatten)]
    pub id: ModId,
    pub slug: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
