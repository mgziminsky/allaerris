use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};
use tokio::fs::{create_dir_all, OpenOptions};

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

// FIXME: Only accept absolute paths
#[cfg(any(not(test), ide))]
impl FsUtils for FsUtil {
    async fn load_file<T>(path: &Path) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let file = OpenOptions::new().read(true).open(path).await?;
        serde_json::from_reader(file.into_std().await).map_err(Into::into)
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
        let file = OpenOptions::new().create(true).write(true).truncate(true).open(path).await?;
        serde_json::to_writer_pretty(file.into_std().await, data).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests_impl {
    use super::*;
    use crate::ErrorKind;

    fn check_path<R: Default>(path: &Path) -> Result<R> {
        if path.starts_with("/pass") {
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
