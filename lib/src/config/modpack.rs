use serde::{Deserialize, Serialize};

use super::Mod;

/// The basic data needed to lookup and install a particular modpack from one of
/// the [supported clients](crate::client)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {
    /// Same as [Mod]
    #[serde(flatten)]
    pub info: Mod,

    /// Whether or not to install overrides contained in the modpack
    pub install_overrides: bool,
}

impl PartialEq for Modpack {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info
    }
}

impl std::ops::Deref for Modpack {
    type Target = Mod;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}
impl std::ops::DerefMut for Modpack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}
