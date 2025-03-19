#![allow(unused_imports)] // Detection broken by cfg...

use std::path::Path;

use anyhow::{Context, anyhow};
use serde::{Serialize, de::DeserializeOwned};
use tokio::fs::{File, create_dir_all};

use crate::Result;

crate::sealed!();

pub trait FsUtils: Sealed {
    #[cfg(any(not(test), ide))]
    async fn load_file<T: DeserializeOwned>(path: &Path) -> Result<T>;
    async fn save_file<T: Serialize>(data: &T, path: &Path) -> Result<()>;

    #[cfg(test)]
    async fn load_file<T: DeserializeOwned + Default>(path: &Path) -> Result<T>;
}

pub struct FsUtil;
impl Sealed for FsUtil {}

#[cfg(any(not(test), ide))]
impl FsUtils for FsUtil {
    async fn load_file<T>(path: &Path) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let file = File::open(path).await?;
        serde_json::from_reader(file.into_std().await)
            .with_context(|| anyhow!("Failed to deserialize file `{}`", path.display()))
            .map_err(Into::into)
    }

    async fn save_file<T>(data: &T, path: &Path) -> Result<()>
    where
        T: Serialize,
    {
        if let Some(path) = path.parent() {
            if !path.exists() {
                create_dir_all(path).await?;
            }
        }
        let file = File::create(path).await?;
        serde_json::to_writer_pretty(file.into_std().await, data)
            .with_context(|| anyhow!("Failed to serialize file `{}`", path.display()))
            .map_err(Into::into)
    }
}

// FIXME: This sucks. How do I make it not suck.
#[cfg(test)]
mod tests_impl {
    use super::*;
    use crate::ErrorKind;

    fn check_path<R: Default>(path: &Path) -> Result<R> {
        if path.iter().any(|c| c == "pass") {
            Ok(R::default())
        } else {
            Err(ErrorKind::TestStub)?
        }
    }

    impl FsUtils for FsUtil {
        async fn load_file<T>(path: &Path) -> Result<T>
        where
            T: DeserializeOwned + Default,
        {
            check_path(path)
        }

        async fn save_file<T>(_: &T, path: &Path) -> Result<()>
        where
            T: Serialize,
        {
            check_path(path)
        }
    }
}
