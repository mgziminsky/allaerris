use std::{borrow::Cow, ffi::OsString, path::PathBuf, sync::LazyLock};

use anyhow::anyhow;
use reqwest::Client;
use url::Url;

use super::maven;
use crate::{config::ModLoader, mgmt::download::Downloadable};


const NAME: &str = "Quilt";
static INSTALLER_URL: LazyLock<Url> = LazyLock::new(|| {
    "https://maven.quiltmc.org/repository/release/org/quiltmc/quilt-installer/maven-metadata.xml"
        .parse()
        .unwrap()
});
static LOADER_URL: LazyLock<Url> = LazyLock::new(|| {
    "https://maven.quiltmc.org/repository/release/org/quiltmc/quilt-loader/maven-metadata.xml"
        .parse()
        .unwrap()
});


pub async fn install(
    super::InstallArgs {
        mngr,
        install_dir,
        version,
    }: super::InstallArgs<'_>,
) -> crate::Result<PathBuf> {
    use super::Version::*;
    let client = Client::builder().build()?;
    let (mc_version, version) = match version {
        Latest(mc_version) => (
            mc_version,
            maven::latest_version(&client, &LOADER_URL, |v| !v.contains('-'))
                .await
                .map(Cow::from)?,
        ),
        Exact(v) => v
            .split_once('+')
            .map(|(v, m)| (m, v.into()))
            .ok_or_else(|| anyhow!("Invalid exact quilt server version: {v}"))?,
    };
    let installer = maven::latest_file(
        NAME,
        &client,
        &INSTALLER_URL,
        mc_version,
        |v| format!("quilt-installer-{v}.jar"),
        |_| true,
    )
    .await?;

    let file = installer
        .download_url()
        .and_then(Url::path_segments)
        .and_then(Iterator::last)
        .expect("quilt installer url valid and includes file");

    super::run_installer(mngr, ModLoader::Quilt, install_dir, file, &installer, |path| {
        let mut cmd = tokio::process::Command::new("java");
        cmd.arg("-jar")
            .arg(path)
            .args(["install", "server", mc_version, &*version])
            .arg("--download-server")
            .arg({
                let mut arg = OsString::from("--install-dir=");
                arg.push(install_dir);
                arg
            });
        cmd
    })
    .await
    .map(|()| install_dir.join("quilt-server-launch.jar"))
}
