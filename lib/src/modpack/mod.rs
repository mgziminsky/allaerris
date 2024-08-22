use async_zip::{
    error::Result,
    tokio::{read::seek::ZipFileReader, write::ZipFileWriter},
    Compression, ZipEntryBuilder,
};
use serde::{Deserialize, Serialize};
use std::{fs::read_dir, path::Path};
use tokio::{
    fs::{canonicalize, create_dir_all, metadata, read, File},
    io::{copy, AsyncBufRead, AsyncSeek, AsyncWrite},
};
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

use crate::read_wrapper;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {}

/// Extract the `input` zip file to `output_dir`
pub async fn extract_zip(
    input: impl AsyncBufRead + AsyncSeek + Unpin,
    output_dir: &Path,
) -> Result<()> {
    let mut zip = ZipFileReader::new(input.compat()).await?;
    for i in 0..zip.file().entries().len() {
        let entry = &zip.file().entries()[i];
        let path = output_dir.join(entry.filename().as_str()?);

        if entry.dir()? {
            create_dir_all(&path).await?;
        } else {
            if let Some(up_dir) = path.parent() {
                if !up_dir.exists() {
                    create_dir_all(up_dir).await?;
                }
            }
            copy(
                &mut zip.reader_without_entry(i).await?.compat(),
                &mut File::create(&path).await?,
            )
            .await?;
        }
    }
    Ok(())
}

/// Compress the input `dir`ectory (starting with `source`) to the given `writer`
///
/// Uses recursion to resolve directories.
/// Resolves symlinks as well.
pub async fn compress_dir<W: AsyncWrite + AsyncSeek + Unpin + Send, P: AsRef<Path> + Send>(
    writer: &mut ZipFileWriter<W>,
    source: &Path,
    dir: P,
    compression: Compression,
) -> Result<()> {
    for entry in read_dir(source.join(dir.as_ref()))? {
        let entry = canonicalize(entry?.path()).await?;
        let meta = metadata(&entry).await?;
        if meta.is_dir() {
            Box::pin(compress_dir(
                writer,
                source,
                &dir.as_ref().join(entry.file_name().unwrap()),
                compression,
            ))
            .await?;
        } else if meta.is_file() {
            let entry_builder = ZipEntryBuilder::new(
                dir.as_ref()
                    .join(entry.file_name().unwrap())
                    .to_string_lossy()
                    .as_ref()
                    .into(),
                compression,
            );
            #[cfg(unix)]
            {
                entry_builder = entry_builder.unix_permissions(
                    std::os::unix::fs::MetadataExt::mode(&meta)
                        .try_into()
                        .unwrap(),
                );
            }
            writer
                .write_entry_whole(entry_builder, &read(entry).await?)
                .await?;
        }
    }
    Ok(())
}

/// Returns the contents of the `file_name` from the provided `input` zip file if it exists
pub async fn read_file_from_zip(
    input: impl AsyncBufRead + AsyncSeek + Unpin,
    file_name: &str,
) -> Result<Option<String>> {
    let zip_file = ZipFileReader::new(input.compat()).await?;
    if let Some(i) = zip_file
        .file()
        .entries()
        .iter()
        .position(|entry| entry.filename().as_str().is_ok_and(|f| f == file_name))
    {
        Ok(Some(
            read_wrapper(zip_file.into_entry(i).await?.compat()).await?,
        ))
    } else {
        Ok(None)
    }
}
