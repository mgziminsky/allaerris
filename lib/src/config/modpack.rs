use serde::{Deserialize, Serialize};

use super::Mod;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {
    #[serde(flatten)]
    pub info: Mod,
    pub install_overrides: bool,
}
