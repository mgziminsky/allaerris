//! Functions for managing server launchers
// Based on https://github.com/nothub/mrpack-install/blob/trunk/server/

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};

use super::{
    cache,
    download::Downloadable,
    events::{EventSouce, ProgressEvent},
    ProfileManager,
};
use crate::{config::ModLoader, ErrorKind};

mod fabric;
mod forge;
mod maven;
mod neoforge;
mod quilt;
mod vanilla;

impl ProfileManager {
    /// Installs the server files for the specified [mod `loader`](ModLoader),
    /// or vanilla
    pub async fn server_install(
        &self,
        loader: Option<ModLoader>,
        version: Version<'_>,
        install_dir: impl AsRef<Path>,
    ) -> crate::Result<PathBuf> {
        let args = InstallArgs {
            mngr: self,
            install_dir: install_dir.as_ref(),
            version,
        };
        if let Some(loader) = loader.and_then(ModLoader::known) {
            use ModLoader::*;
            match loader {
                Fabric => fabric::install(args).await,
                Forge => forge::install(args).await,
                NeoForge => neoforge::install(args).await,
                Quilt => quilt::install(args).await,
                Unknown => unreachable!(),
                l => Err(ErrorKind::ServerUnsupported(l).into()),
            }
        } else {
            vanilla::install(args).await
        }
    }
}

async fn install_file(
    mngr: &ProfileManager,
    loader: Option<ModLoader>,
    install_dir: &Path,
    file: &str,
    meta: &dyn Downloadable,
) -> crate::Result<PathBuf> {
    let install_path = install_dir.join(file);
    let cache_path = cache::server_path(loader).join(file);
    let success = mngr
        .download(meta, if mngr.no_cache { install_path.as_ref() } else { cache_path.as_ref() })
        .await
        .is_some();

    if success {
        if !mngr.no_cache {
            use tokio::fs;
            fs::create_dir_all(install_dir)
                .await
                .with_context(|| install_dir.display().to_string())?;
            let _ = fs::copy(&cache_path, &install_path)
                .await
                .with_context(move || cache_path.display().to_string())
                .context("Failed to copy server file into install directory")?;
        }
        Ok(install_path)
    } else {
        Err(ErrorKind::DownloadFailed(meta.download_url().cloned().unwrap()).into())
    }
}

async fn run_installer(
    mngr: &ProfileManager,
    loader: ModLoader,
    install_dir: &Path,
    file: &str,
    dl: &dyn Downloadable,
    cmd: impl Fn(&Path) -> tokio::process::Command,
) -> crate::Result<()> {
    let save_path = &if mngr.no_cache {
        install_dir.join(file)
    } else {
        cache::server_path(Some(loader)).join(file).into()
    };
    if mngr.download(dl, save_path).await.is_some() {
        mngr.send(ProgressEvent::Status("Running server installer...".to_owned()));
        let status = cmd(save_path).status().await?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Installer failed with exit code [{status}]").into())
        }
    } else {
        Err(ErrorKind::DownloadFailed(dl.download_url().cloned().unwrap()).into())
    }
}


/// The server version to install. Both variants are treated as "exact" for
/// vanilla
pub enum Version<'a> {
    /// latest server version built for the specified MC version
    Latest(&'a str),

    /// Exact server version to be installed
    Exact(&'a str),
}

struct InstallArgs<'a> {
    mngr: &'a ProfileManager,
    install_dir: &'a Path,
    version: Version<'a>,
}
