use core::task;
use std::{
    io::{self, Write},
    path::Path,
    pin::Pin,
    task::Poll,
};

use anyhow::anyhow;
use sha1::{Digest, Sha1};
use tokio::{fs::File, io::AsyncWrite};

use crate::Result;

pub fn hex_decode(str: &str) -> Result<impl AsRef<[u8]>> {
    if str.len() & 1 != 0 {
        return Err(anyhow!("Hex string length must be even").into());
    }
    let steps = (0..str.len()).step_by(2);
    let mut bytes = Vec::with_capacity(steps.len());
    for i in steps {
        bytes.push(u8::from_str_radix(&str[i..i + 2], 16).map_err(anyhow::Error::new)?);
    }
    Ok(bytes)
}

pub async fn verify_sha1(sha1: &str, path: &Path) -> Result<bool> {
    let sha1 = hex_decode(sha1)?;

    let mut hasher = Sha1Async(Sha1::new());
    let mut file = File::open(path).await?;
    tokio::io::copy(&mut file, &mut hasher).await?;

    Ok(sha1.as_ref() == &*hasher.0.finalize())
}

struct Sha1Async(Sha1);
impl AsyncWrite for Sha1Async {
    fn poll_write(self: Pin<&mut Self>, _: &mut task::Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        Poll::Ready(self.get_mut().0.write(buf))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(self.get_mut().0.flush())
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
