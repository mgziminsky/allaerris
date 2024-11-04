use std::{borrow::Cow, path::PathBuf, sync::LazyLock};

use anyhow::anyhow;
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::mgmt::download::Downloadable;


static MANIFEST_URL: LazyLock<Url> = LazyLock::new(|| "https://launchermeta.mojang.com/mc/game/version_manifest.json".parse().unwrap());


pub async fn install(
    super::InstallArgs {
        mngr,
        install_dir,
        version,
    }: super::InstallArgs<'_>,
) -> crate::Result<PathBuf> {
    use super::Version::*;
    let version = match version {
        Latest(mc_version) => mc_version,
        Exact(v) => v,
    };

    let client = Client::builder().build()?;

    let Manifest { versions } = client.get(MANIFEST_URL.clone()).send().await?.json().await?;
    let details = versions
        .into_iter()
        .find(|v| v.id == version)
        .ok_or_else(|| anyhow!("Unknown vanilla server version: {version}"))?;
    let meta: Meta = client.get(details.url).send().await?.json().await?;

    let file = format!("server-vanilla-{version}.jar");
    super::install_file(mngr, None, install_dir, &file, &meta).await
}


#[derive(Debug, Deserialize)]
struct Manifest {
    versions: Vec<Version>,
}
#[derive(Debug, Deserialize)]
struct Version {
    id: String,
    url: Url,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Type {
    Release,
    Snapshot,

    #[serde(other)]
    Other,
}


#[derive(Debug, Deserialize)]
struct Meta {
    id: String,
    downloads: Downloads,
}
#[derive(Debug, Deserialize)]
struct Downloads {
    server: Download,
}
#[derive(Debug, Deserialize)]
struct Download {
    sha1: String,
    size: u64,
    url: Url,
}

impl Downloadable for Meta {
    fn id(&self) -> crate::mgmt::events::DownloadId {
        u64::from_str_radix(&self.downloads.server.sha1[..16], 16)
            .unwrap_or(self.length())
            .into()
    }

    fn download_url(&self) -> Option<&Url> {
        Some(&self.downloads.server.url)
    }

    fn title(&self) -> Cow<str> {
        format!("Vanilla Server {}", self.id).into()
    }

    fn length(&self) -> u64 {
        self.downloads.server.size
    }

    fn sha1(&self) -> Option<&str> {
        Some(&self.downloads.server.sha1)
    }
}
