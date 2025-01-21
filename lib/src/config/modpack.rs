use serde::{Deserialize, Serialize};

use super::Mod;
use crate::client::schema::{Project, ProjectType};

/// The basic data needed to lookup and install a particular modpack from one of
/// the [supported clients](crate::client)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {
    /// Same as [Mod]
    #[serde(flatten)]
    info: Mod,

    /// Whether or not to install overrides contained in the modpack
    pub install_overrides: bool,
}

impl Modpack {
    /// Creates a new config [`Modpack`] from a [`Project`]
    ///
    /// # Panics
    /// If the [`project_type`](ProjectType) is not
    /// [`ModPack`](ProjectType::ModPack). Use [`Modpack::try_new`] for a safe
    /// version.
    pub fn new(proj: Project, install_overrides: bool) -> Self {
        assert!(
            proj.project_type == ProjectType::ModPack,
            "Tried to create Modpack from project of wrong type: {:?}",
            proj.project_type
        );
        Self {
            info: proj.into(),
            install_overrides,
        }
    }

    /// Creates a new config [`Modpack`] from a [`Project`]
    ///
    /// # Errors
    /// Returns the passed in project in the [`Err`] if the [`project_type`] is
    /// not [`ModPack`](ProjectType::ModPack). Since [`Project`] is large, it
    /// will be expensive to return. If you still need the project in the error
    /// case, the caller should check the [`project_type`] before calling and
    /// handle the error case preemptively.
    ///
    /// [`project_type`]: ProjectType
    #[allow(clippy::result_large_err)]
    pub fn try_new(proj: Project, install_overrides: bool) -> Result<Self, Project> {
        if proj.project_type != ProjectType::ModPack {
            return Err(proj);
        }
        Ok(Self {
            info: proj.into(),
            install_overrides,
        })
    }
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
