use std::{
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::LazyLock,
};

use anyhow::anyhow;
use reqwest::Client;
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use url::Url;

use crate::{
    config::ModLoader,
    mgmt::{download::Downloadable, events::DownloadId},
};


macro_rules! fabric_url {
    ($path:literal) => {
        concat!("https://meta.fabricmc.net/v2/versions/", $path).parse().unwrap()
    };
}
static INSTALLER_URL: LazyLock<Url> = LazyLock::new(|| fabric_url!("installer/"));
static LOADER_URL: LazyLock<Url> = LazyLock::new(|| fabric_url!("loader/"));


pub async fn install(
    super::InstallArgs {
        mngr,
        install_dir,
        version,
    }: super::InstallArgs<'_>,
) -> crate::Result<PathBuf> {
    let meta = installer_meta(version).await?;
    let file = format!(
        "fabric-server-mc.{}-loader.{}-launcher.{}.jar",
        meta.mc_version, meta.version, meta.installer
    );
    super::install_file(mngr, Some(ModLoader::Fabric), install_dir, &file, &meta).await
}

async fn installer_meta(version: super::Version<'_>) -> crate::Result<FabricServer<'_>> {
    use super::Version::*;
    let client = Client::builder().build()?;

    let LatestVersion(VersionMeta { version: installer, .. }) = client.get(INSTALLER_URL.clone()).send().await?.json().await?;
    let (mc_version, version) = match version {
        Latest(mc_version) => (
            mc_version,
            { client.get(LOADER_URL.clone()).send().await?.json::<LatestVersion>().await?.0 }
                .version
                .into(),
        ),
        Exact(v) => v
            .split_once('+')
            .map(|(m, v)| (m, Cow::from(v)))
            .ok_or_else(|| anyhow!("Invalid exact fabric server version: {v}"))?,
    };

    let url = LOADER_URL
        .join(&format!("{mc_version}/{version}/{installer}/server/jar"))
        .map_err(anyhow::Error::new)?;

    Ok(FabricServer {
        id: {
            let mut hasher = DefaultHasher::new();
            url.hash(&mut hasher);
            hasher.finish().into()
        },
        title: format!("Fabric Server {mc_version}+{version}"),
        url,
        mc_version,
        version,
        installer,
    })
}

#[derive(Debug)]
struct FabricServer<'a> {
    id: DownloadId,
    title: String,
    url: Url,
    mc_version: &'a str,
    version: Cow<'a, str>,
    installer: String,
}
impl Downloadable for FabricServer<'_> {
    fn id(&self) -> DownloadId {
        self.id
    }

    fn download_url(&self) -> Option<&Url> {
        Some(&self.url)
    }

    fn title(&self) -> std::borrow::Cow<str> {
        self.title.as_str().into()
    }

    fn length(&self) -> u64 {
        0
    }

    fn sha1(&self) -> Option<&str> {
        None
    }
}


#[derive(Debug, Deserialize)]
struct VersionMeta {
    stable: bool,
    version: String,
}

struct LatestVersion(VersionMeta);
impl std::ops::Deref for LatestVersion {
    type Target = VersionMeta;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'de> Deserialize<'de> for LatestVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LatestVisitor;
        impl<'de> Visitor<'de> for LatestVisitor {
            type Value = LatestVersion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a nonempty sequence of versions")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut max = None::<VersionMeta>;
                while let Some::<VersionMeta>(meta) = seq.next_element()? {
                    if meta.stable && (max.is_none() || max.as_ref().is_some_and(|max| meta.version > max.version)) {
                        max.replace(meta);
                    }
                }
                max.map(LatestVersion).ok_or(de::Error::custom("no versions"))
            }
        }

        deserializer.deserialize_seq(LatestVisitor)
    }
}
