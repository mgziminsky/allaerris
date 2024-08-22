use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModId {
    Forge(usize),
    Modrinth(String),
    GitHub((String, String)),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    pub id: ModId,
    pub slug: String,
    pub name: String,
}
