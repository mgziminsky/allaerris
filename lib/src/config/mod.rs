use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt},
};

mod loader;
mod modpack;
mod mods;
mod profile;

pub use loader::*;
pub use modpack::*;
pub use mods::*;
pub use profile::*;

use crate::{Result, CONF_DIR};

pub static DEFAULT_CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    CONF_DIR.join(
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.with_extension("json").file_name().map(ToOwned::to_owned))
            .unwrap_or("config.json".into()),
    )
});

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    /// The index of the active profile
    #[serde(default, skip_serializing_if = "is_zero")]
    pub active_profile: usize,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
}

impl Config {
    pub fn active_profile(&self) -> Option<&Profile> {
        self.profiles.get(self.active_profile)
    }

    pub fn active_profile_mut(&mut self) -> Option<&mut Profile> {
        self.profiles.get_mut(self.active_profile)
    }

    /// Returns the enumeration of all profiles with an associated modpack.
    pub fn modpacks(&self) -> impl Iterator<Item = (usize, &Profile)> {
        self.profiles.iter().enumerate().filter(|(_, p)| p.modpack.is_some())
    }
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

async fn open_config_file(path: impl AsRef<Path>) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .create(true)
        .open(path)
        .await
        .map_err(Into::into)
}

/// Open the config file at `path`.
/// If it doesn't exist, a config file with an empty config will be created and opened.
pub async fn get_file(path: impl AsRef<Path>) -> Result<File> {
    let path = path.as_ref();
    if path.exists() {
        open_config_file(path).await
    } else {
        create_dir_all(path.parent().unwrap()).await?;
        let mut file = open_config_file(path).await?;
        write_file(&mut file, &Config::default()).await?;
        Ok(file)
    }
}

/// Serialise `config` and write it to `config_file`
pub async fn write_file(config_file: &mut File, config: &Config) -> Result<()> {
    let serialised = serde_json::to_string_pretty(config)?;
    config_file.set_len(0).await?; // Clear the file contents
    config_file.rewind().await?; // Set the cursor to the beginning
    config_file.write_all(serialised.as_bytes()).await?;
    config_file.rewind().await?; // So that subsequent reads work properly
    Ok(())
}
