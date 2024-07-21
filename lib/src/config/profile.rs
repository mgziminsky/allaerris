mod by_path;
mod data;

use std::io;

use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

pub(super) use self::by_path::ProfileByPath;
pub use self::data::*;
use crate::{config::Mod, ErrorKind, PathAbsolute, Result};

fn name_lowercase(m: &Mod) -> String {
    m.name.to_lowercase()
}


/// A lazy loaded profile containing the `name` and `path`. The external
/// [profile data](ProfileData) will be loaded and cached on first access.
/// Path is immutable after creation since it is used as the profile id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub(super) name: String,
    pub(super) path: PathAbsolute,

    #[serde(skip)]
    data: OnceCell<ProfileData>,
}

impl Profile {
    /// Creates a new [`Profile`].
    pub fn new(name: String, path: PathAbsolute) -> Self {
        Self {
            name,
            path,
            data: OnceCell::new(),
        }
    }

    /// Create a new profile with the given `name`, `path`, and
    /// [`data`](ProfileData)
    pub fn with_data(name: String, path: PathAbsolute, data: ProfileData) -> Self {
        Self {
            name,
            path,
            data: data.into(),
        }
    }

    /// The [path](std::path::Path) to the root of this profile.
    ///
    /// This path is where mods/modpacks will be installed, and where the
    /// [profile data](ProfileData) is stored in a file named
    #[doc = concat!('`', data::consts!(FILENAME), '`')]
    pub fn path(&self) -> &PathAbsolute {
        &self.path
    }

    /// The profile name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the profile name
    pub fn set_name(&mut self, name: &impl ToString) {
        self.name = name.to_string();
    }

    /// Will attempt to load the [profile data](ProfileData) located at this
    /// profile's [path](Self::path) on first access and return it. Subsequent
    /// calls will return the previously loaded data without accessing the
    /// filesystem again
    ///
    /// # Errors
    ///
    /// This function will return an error if loading the data from the
    /// filesystem fails
    pub async fn data(&self) -> Result<&ProfileData> {
        self.data
            .get_or_try_init(|| async {
                let p = match ProfileData::load(&self.path).await {
                    Ok(p) => p,
                    Err(e) if matches!(e.kind(), ErrorKind::IO(e) if e.kind() == io::ErrorKind::NotFound) => ProfileData::default(),
                    err => return err,
                };
                Ok(p)
            })
            .await
    }

    /// See [`data`](Self::data)
    #[allow(clippy::missing_panics_doc)]
    pub async fn data_mut(&mut self) -> Result<&mut ProfileData> {
        self.data().await?; // Ensure data is initialized
        Ok(self.data.get_mut().expect("data should always have been initialized by here"))
    }

    /// Sorts [`mods`] list by name then save the [profile data](ProfileData) to
    /// the filesystem at [`path`](Self::path)
    ///
    /// [`mods`]: ProfileData::mods
    /// # Errors
    ///
    /// Will return any IO errors encountered while attempting to save to the
    /// filesystem
    pub async fn save(&mut self) -> Result<()> {
        if let Some(data) = self.data.get_mut() {
            data.mods.sort_by_cached_key(name_lowercase);
            data.save_to(&self.path).await?;
        }
        Ok(())
    }

    /// Returns true if the [data](ProfileData) file for this [`Profile`]
    /// exists.
    pub fn exists(&self) -> bool {
        ProfileData::file_path(&self.path).exists()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::schema::ProjectId;

    #[test]
    fn save_sort() {
        let unsorted = [
            Mod {
                id: ProjectId::Forge(3).into(),
                slug: "test-3".to_owned(),
                name: "Test 3".to_owned(),
            },
            Mod {
                id: ProjectId::Forge(1).into(),
                slug: "test-1".to_owned(),
                name: "test 1".to_owned(),
            },
            Mod {
                id: ProjectId::Forge(2).into(),
                slug: "test-2".to_owned(),
                name: "Test 2".to_owned(),
            },
            Mod {
                id: ProjectId::Forge(0).into(),
                slug: "test-0".to_owned(),
                name: "test 0".to_owned(),
            },
        ];
        let sorted = {
            let mut s = unsorted.clone();
            s.sort_by_cached_key(name_lowercase);
            s
        };
        dbg!(&sorted);
        let mut p = Profile::new("Test Profile".to_string(), "/dev/null/pass".parse().unwrap());
        crate::block_on(p.data_mut())
            .expect("Load should use defaults")
            .mods
            .extend(unsorted);
        assert!(crate::block_on(p.save()).is_ok(), "Save operation should have succeeded");
        assert_eq!(
            sorted.as_slice(),
            crate::block_on(p.data()).unwrap().mods,
            "Mods should be sorted after save"
        );
    }
}
