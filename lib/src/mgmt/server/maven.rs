use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::{Context, anyhow};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::mgmt::{download::Downloadable, events::DownloadId};


const SHA1_EXT: &str = ".sha1";


fn meta_versions(xml: impl AsRef<str>) -> crate::Result<Vec<String>> {
    let meta: Meta = quick_xml::de::from_str(xml.as_ref()).map_err(anyhow::Error::new)?;
    Ok(meta.versioning.versions.version)
}

pub async fn latest_version(client: &Client, meta_url: &Url, filter: impl FnMut(&String) -> bool) -> crate::Result<String> {
    meta_versions(client.get(meta_url.clone()).send().await?.text().await?)?
        .into_iter()
        .filter(filter)
        .max()
        .ok_or(anyhow!("No matching version").into())
}

pub async fn latest_file(
    name: &str,
    client: &Client,
    meta_url: &Url,
    mc_version: &str,
    mut version_file: impl FnMut(&str) -> String,
    version_pred: impl FnMut(&String) -> bool,
) -> crate::Result<MavenFile> {
    let version = latest_version(client, meta_url, version_pred)
        .await
        .with_context(|| anyhow!("No {name} server found for MC version `{mc_version}`"))?;
    exact_file(name, client, meta_url, &version, version_file(&version)).await
}

pub async fn exact_file(name: &str, client: &Client, meta_url: &Url, exact_version: &str, file_name: String) -> crate::Result<MavenFile> {
    let mut url = meta_url.clone();

    url.path_segments_mut().unwrap().pop().push(exact_version).push(&file_name);
    let length = client.head(url.clone()).send().await?.content_length().unwrap_or_default();

    let sha1 = {
        let mut url = url.clone();
        url.path_segments_mut().unwrap().pop().extend(Some(file_name + SHA1_EXT));
        client.get(url).send().await?.text().await.ok()
    };

    Ok(MavenFile {
        id: {
            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            hasher.finish().into()
        },
        title: format!("{name} Server Installer {exact_version}"),
        url,
        length,
        sha1,
    })
}


#[derive(Debug, Deserialize)]
struct Meta {
    versioning: Versioning,
}
#[derive(Debug, Deserialize)]
struct Versioning {
    versions: Versions,
}
#[derive(Debug, Deserialize)]
struct Versions {
    #[serde(default)]
    version: Vec<String>,
}


#[derive(Debug)]
pub(super) struct MavenFile {
    id: DownloadId,
    url: Url,
    title: String,
    length: u64,
    sha1: Option<String>,
}
impl Downloadable for MavenFile {
    fn id(&self) -> DownloadId {
        self.id
    }

    fn download_url(&self) -> Option<&Url> {
        Some(&self.url)
    }

    fn title(&self) -> std::borrow::Cow<'_, str> {
        self.title.as_str().into()
    }

    fn length(&self) -> u64 {
        self.length
    }

    fn sha1(&self) -> Option<&str> {
        self.sha1.as_deref()
    }
}
