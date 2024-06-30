use github::models::AssetId;
use url::Url;

use super::ProjectId;
use crate::client::service_id::svc_id_impl;

svc_id_impl! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum VersionId {
        Forge(u64),
        Modrinth(String),
        Github(AssetId),
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub id: VersionId,
    pub project_id: ProjectId,
    pub title: String,
    pub download_url: Option<Url>,
    pub filename: String,
    pub length: u64,
    pub date: String,
    pub sha1: Option<String>,
    pub deps: Vec<Dependency>,
}

/* impl ModFile {
    /// Consumes `self` and downloads the file to the `output_dir`.
    ///
    /// The `update` closure is called with the chunk length whenever a chunk is downloaded and written.
    ///
    /// Returns the size of the file and the filename
    pub async fn download<UF>(
        &self,
        client: &Client,
        output_dir: &Path,
        mut update: UF,
    ) -> Result<PathBuf>
    where
        UF: FnMut(usize) + Send,
    {
        use tokio::{
            fs::{create_dir_all, rename, OpenOptions},
            io::{AsyncWriteExt, BufWriter},
        };

        let out_file_path = output_dir.join(&self.filename);
        let temp_file_path = out_file_path.with_extension("part");
        if let Some(up_dir) = out_file_path.parent() {
            create_dir_all(up_dir).await?;
        }

        let mut temp_file = BufWriter::with_capacity(
            self.length,
            OpenOptions::new()
                .create(true)
                .open(&temp_file_path)
                .await?,
        );

        let mut response = client.get(self.download_url.as_ref()).send().await?;

        while let Some(chunk) = response.chunk().await? {
            temp_file.write_all(&chunk).await?;
            update(chunk.len());
        }
        temp_file.shutdown().await?;
        rename(temp_file_path, &out_file_path).await?;
        Ok(out_file_path)
    }
} */

#[derive(Debug, Clone)]
pub struct Dependency {
    pub project_id: ProjectId,
    pub id: Option<VersionId>,
    pub dep_type: DependencyType,
}

#[derive(Debug, Clone, Copy)]
pub enum DependencyType {
    Required,
    Optional,
    Other,
}
