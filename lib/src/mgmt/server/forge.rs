use std::{path::PathBuf, sync::LazyLock};

use reqwest::Client;
use url::Url;

use super::maven;
use crate::{config::ModLoader, mgmt::download::Downloadable};


const NAME: &str = "Forge";
static META_URL: LazyLock<Url> = LazyLock::new(|| {
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml"
        .parse()
        .unwrap()
});


fn version_file(version: &str) -> String {
    format!("forge-{version}-installer.jar")
}

pub async fn install(
    super::InstallArgs {
        mngr,
        install_dir,
        version,
    }: super::InstallArgs<'_>,
) -> crate::Result<PathBuf> {
    use super::Version::*;
    let client = Client::builder().build()?;
    let installer = match version {
        Latest(mc_version) => {
            maven::latest_file(NAME, &client, &META_URL, mc_version, version_file, |v| {
                v.split_once('-').is_some_and(|(mc, _)| mc == mc_version)
            })
            .await
        },
        Exact(v) => maven::exact_file(NAME, &client, &META_URL, v, version_file(v)).await,
    }?;

    let file = installer
        .download_url()
        .and_then(Url::path_segments)
        .and_then(Iterator::last)
        .expect("forge installer url valid and includes file");
    super::run_installer(mngr, ModLoader::Forge, install_dir, file, &installer, |path| {
        let mut cmd = tokio::process::Command::new("java");
        cmd.arg("-jar").arg(path).arg("--installServer").arg(install_dir);
        cmd
    })
    .await
    .map(|()| install_dir.join(const { if cfg!(windows) { "run.bat" } else { "run.sh" } }))
}
