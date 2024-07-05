use std::convert::identity;

use anyhow::{anyhow, Context};
use sha1::{Digest, Sha1};
use tokio::{fs::File, io::AsyncWriteExt};

use super::hash::{hex_decode, verify_sha1};
use crate::{checked_types::PathAbsolute, client::schema::Version, ErrorKind, Result, CACHE_DIR};


pub async fn dl_version(mut v: Version) -> Result<(Version, PathAbsolute)> {
    let cache_path = {
        let mut path = CACHE_DIR.join("mods");
        path.push(v.project_id.to_string());
        path.push(v.id.to_string());
        path.push(v.filename.file_name().expect("version should always contain a file name"));
        path
    };
    if cache_path.is_file() && v.sha1.is_some() && verify_sha1(v.sha1.as_ref().unwrap(), &cache_path).await.is_ok_and(identity) {
        return Ok((v, cache_path));
    }

    if let Some(url) = &v.download_url {
        let sha1 = dl_verified(&cache_path, v.sha1.as_ref(), url.clone())
            .await
            .with_context(|| ErrorKind::DownloadFailed(v.project_id.clone(), url.clone()))?;
        v.sha1 = Some(sha1);

        Ok((v, cache_path))
    } else {
        Err(ErrorKind::DistributionDenied(v.project_id, v.title).into())
    }
}

async fn dl_verified(cache_path: &PathAbsolute, sha1: Option<&impl AsRef<str>>, url: url::Url) -> Result<String> {
    async fn _dl_verified(cache_path: &PathAbsolute, sha1: Option<&str>, url: url::Url) -> Result<String> {
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let temp_path = {
            let mut tmp = cache_path.to_path_buf();
            tmp.as_mut_os_string().push(".part");
            tmp
        };
        let mut resp = reqwest::get(url).await?;
        let mut file = File::create(&temp_path).await?;
        let mut hasher = Sha1::new();
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            hasher.update(&chunk);
        }
        file.flush().await?;

        let computed = hasher.finalize();
        let sha_bytes = sha1.and_then(|s| hex_decode(s).ok());
        if sha_bytes.is_none() || sha_bytes.unwrap().as_ref() == &*computed {
            tokio::fs::rename(temp_path, cache_path).await?;
            Ok(format!("{:x}", computed))
        } else {
            Err(anyhow!(
                "Incorrect hash for downloaded file:\n\tExpected: {}\n\t  Actual: {computed:x}",
                sha1.expect("sha1 should not be none")
            )
            .into())
        }
    }
    _dl_verified(cache_path, sha1.map(AsRef::as_ref), url).await
}
