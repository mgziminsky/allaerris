use std::convert::identity;

use anyhow::{anyhow, Context};
use sha1::{Digest, Sha1};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    checked_types::PathAbsolute,
    client::schema::Version,
    mgmt::{
        events::{DownloadProgress, EventSouce, ProjectIdHash},
        hash::{hex_decode, verify_sha1},
        ProfileManager,
    },
    ErrorKind, Result, CACHE_DIR,
};


impl ProfileManager {
    pub(super) async fn dl_version(&self, mut v: Version) -> Option<(Version, PathAbsolute)> {
        let phash = (&v.project_id).into();
        self.send(
            DownloadProgress::Start {
                project: phash,
                title: v.title.clone(),
                length: v.length,
            }
            .into(),
        );
        let cache_path = {
            let mut path = CACHE_DIR.join("mods");
            path.push(v.project_id.to_string());
            path.push(v.id.to_string());
            path.push(v.filename.file_name().expect("version should always contain a file name"));
            path
        };
        if cache_path.is_file() && v.sha1.is_some() && verify_sha1(v.sha1.as_ref().unwrap(), &cache_path).await.is_ok_and(identity) {
            self.send(DownloadProgress::Success(phash).into());
            return Some((v, cache_path));
        }

        if let Some(url) = &v.download_url {
            match self
                .dl_verified(phash, &cache_path, v.sha1.as_ref(), url.clone())
                .await
                .with_context(|| ErrorKind::DownloadFailed(v.project_id.clone(), url.clone()))
            {
                Ok(sha1) => {
                    v.sha1 = Some(sha1);
                    self.send(DownloadProgress::Success(phash).into());
                    Some((v, cache_path))
                },
                Err(e) => {
                    self.send(DownloadProgress::Fail(phash, e.into()).into());
                    None
                },
            }
        } else {
            self.send(DownloadProgress::Fail(phash, ErrorKind::DistributionDenied(v.project_id, v.title).into()).into());
            None
        }
    }

    async fn dl_verified(&self, phash: ProjectIdHash, cache_path: &PathAbsolute, sha1: Option<&String>, url: url::Url) -> Result<String> {
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
            self.send(DownloadProgress::Progress(phash, chunk.len() as _).into());
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
}
