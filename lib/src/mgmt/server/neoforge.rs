use std::{borrow::Cow, path::PathBuf, sync::LazyLock};

use reqwest::Client;
use url::Url;

use super::maven;
use crate::{config::ModLoader, mgmt::download::Downloadable};


const NAME: &str = "NeoForge";
static META_URL: LazyLock<Url> = LazyLock::new(|| {
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml"
        .parse()
        .unwrap()
});


fn version_file(version: &str) -> String {
    format!("neoforge-{version}-installer.jar")
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
            // https://docs.neoforged.net/docs/gettingstarted/versioning#neoforge
            let major_minor = mc_version
                .strip_prefix("1.")
                .map(Cow::from)
                .map_or_else(|| mc_version.into(), |v| if v.contains('.') { v } else { format!("{v}.0").into() });
            maven::latest_file(NAME, &client, &META_URL, mc_version, version_file, move |v| {
                v.rsplit_once('.').is_some_and(|(mc, _)| mc == major_minor)
            })
            .await
        },
        Exact(v) => maven::exact_file(NAME, &client, &META_URL, v, version_file(v)).await,
    }?;

    let file = installer
        .download_url()
        .and_then(Url::path_segments)
        .and_then(Iterator::last)
        .expect("neoforge installer url valid and includes file");

    super::run_installer(mngr, ModLoader::NeoForge, install_dir, file, &installer, |path| {
        let mut cmd = tokio::process::Command::new("java");
        cmd.arg("-jar").arg(path).arg("--install-server").arg(install_dir);
        cmd
    })
    .await
    .map(|()| {
        install_dir.join(
            const {
                if cfg!(windows) {
                    "run.bat"
                } else {
                    "run.sh"
                }
            },
        )
    })
}
