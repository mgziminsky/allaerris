mod by_path;
mod data;

use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

pub(crate) use self::by_path::ProfileByPath;
pub use self::data::{ProfileData, DEFAULT_GAME_VERSION};
use crate::{config::Mod, ErrorKind, Result};

fn name_lowercase(m: &Mod) -> String {
    m.name.to_lowercase()
}


/// A lazy loaded profile containing the `name` and `path`. The external
/// [profile data](ProfileData) will be loaded and cached on first access.
/// Path is immutable after creation since it is used as the profile id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub(super) name: String,
    pub(super) path: PathBuf,
    #[serde(skip)]
    data: OnceCell<ProfileData>,
}

macro_rules! check_path {
    ($path:ident) => {
        if $path.as_os_str().to_string_lossy().trim().is_empty() || $path.is_relative() {
            return Err(ErrorKind::PathInvalid)?;
        }
    };
}
impl Profile {
    pub(super) fn new_unchecked(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            data: OnceCell::new(),
        }
    }

    /// Create a new profile with the given `name` and `path`
    /// # Errors
    ///
    /// Will return an error if `path` is empty or whitespace only
    pub fn new(name: String, path: PathBuf) -> Result<Self> {
        check_path!(path);
        Ok(Self::new_unchecked(name, path))
    }

    /// Create a new profile with the given `name`, `path`, and
    /// [`data`](ProfileData)
    /// # Errors
    ///
    /// Will return an error if `path` is empty or whitespace only
    pub fn with_data(name: String, path: PathBuf, data: ProfileData) -> Result<Self> {
        check_path!(path);
        Ok(Self {
            name,
            path,
            data: data.into(),
        })
    }

    /// The [path](Path) to the root of this profile.
    ///
    /// This path is where mods/modpacks will be installed, and where the
    /// [profile data](ProfileData) is stored in a file named
    #[doc = concat!('`', data::consts!(FILENAME), '`')]
    pub fn path(&self) -> &Path {
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

    /// Returns true if the [data](ProfileData) file for this [`Profile`] exists.
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
                id: ProjectId::Forge(3),
                slug: "test-3".to_owned(),
                name: "Test 3".to_owned(),
            },
            Mod {
                id: ProjectId::Forge(1),
                slug: "test-1".to_owned(),
                name: "test 1".to_owned(),
            },
            Mod {
                id: ProjectId::Forge(2),
                slug: "test-2".to_owned(),
                name: "Test 2".to_owned(),
            },
            Mod {
                id: ProjectId::Forge(0),
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
        let mut p = Profile::new_unchecked("Test Profile".to_string(), "/pass/null".into());
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
