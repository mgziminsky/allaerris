use std::{borrow::Cow, convert::identity, path::Path};

use anyhow::{Context, anyhow};
use sha1::{Digest, Sha1};
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

use crate::{
    ErrorKind, Result,
    client::schema::Version,
    hash::{hex_decode, verify_sha1},
    mgmt::{
        ProfileManager,
        events::{DownloadId, DownloadProgress, EventSouce},
    },
};

pub trait Downloadable: Sync {
    /// A unique id for identifying this download
    fn id(&self) -> DownloadId;
    fn download_url(&self) -> Option<&Url>;
    fn title(&self) -> Cow<'_, str>;
    fn length(&self) -> u64;
    fn sha1(&self) -> Option<&str>;
}


impl ProfileManager {
    pub(in crate::mgmt) async fn download(&self, dl: &dyn Downloadable, save_path: &Path) -> Option<String> {
        // Limit number of concurrent downloads.
        // Should this be configurable? Maybe via an environment variable
        static PERMITS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(10);
        let _permit = PERMITS.acquire().await.unwrap();

        let id = dl.id();
        let title = dl.title();
        self.send(
            DownloadProgress::Start {
                project: id,
                title: (*title).to_owned(),
                length: dl.length(),
            }
            .into(),
        );
        let sha1 = dl.sha1();
        if !self.force && save_path.is_file() && sha1.is_some() && verify_sha1(sha1.unwrap(), save_path).await.is_ok_and(identity) {
            self.send(DownloadProgress::Success(id).into());
            return sha1.map(Into::into);
        }

        if let Some(url) = dl.download_url() {
            match self
                .dl_verified(id, save_path, sha1, url.clone())
                .await
                .with_context(|| ErrorKind::DownloadFailed(url.clone()))
            {
                Ok(sha1) => {
                    self.send(DownloadProgress::Success(id).into());
                    Some(sha1)
                },
                Err(e) => {
                    self.send(DownloadProgress::Fail(id, e.into()).into());
                    None
                },
            }
        } else {
            self.send(DownloadProgress::Fail(id, ErrorKind::DistributionDenied(title.into_owned()).into()).into());
            None
        }
    }

    async fn dl_verified(&self, dlid: DownloadId, out_path: &Path, sha1: Option<&str>, url: Url) -> Result<String> {
        if let Some(parent) = out_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let temp_path = {
            let mut tmp = out_path.to_path_buf();
            tmp.as_mut_os_string().push(".part");
            tmp
        };
        let mut resp = reqwest::get(url).await?;
        let mut file = File::create(&temp_path).await?;
        let mut hasher = Sha1::new();
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            self.send(DownloadProgress::Progress(dlid, chunk.len() as _).into());
            hasher.update(&chunk);
        }
        file.flush().await?;

        let computed = hasher.finalize();
        let sha_bytes = sha1.and_then(|s| hex_decode(s).ok());
        if sha_bytes.is_none() || sha_bytes.unwrap().as_ref() == &*computed {
            tokio::fs::rename(temp_path, out_path).await?;
            Ok(format!("{computed:x}"))
        } else {
            Err(anyhow!(
                "Incorrect hash for downloaded file:\n\tExpected: {}\n\t  Actual: {computed:x}",
                sha1.expect("sha1 should not be none")
            )
            .into())
        }
    }
}

impl Downloadable for Version {
    fn id(&self) -> DownloadId {
        (&self.project_id).into()
    }

    fn download_url(&self) -> Option<&Url> {
        self.download_url.as_ref()
    }

    fn title(&self) -> Cow<'_, str> {
        self.title.as_str().into()
    }

    fn length(&self) -> u64 {
        self.length
    }

    fn sha1(&self) -> Option<&str> {
        self.sha1.as_deref()
    }
}
