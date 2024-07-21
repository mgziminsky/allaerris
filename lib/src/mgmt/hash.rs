use core::task;
use std::{
    io::{self, Write},
    path::Path,
    pin::Pin,
    task::Poll,
};

use anyhow::anyhow;
use sha1::{digest::Output, Digest, Sha1};
use tokio::io::AsyncWrite;

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

macro_rules! verify_impl {
    ($($name:ident($hasher:expr) = $use:ident $(@ $as:ident / $aw:ident)? );+$(;)?) => {$(
        pub $($as)? fn $name(sha1: &str, path: &Path) -> Result<bool> {
            let sha1 = hex_decode(sha1)?;

            let mut hasher = $hasher;
            let mut file = $use::fs::File::open(path)$(.$aw)??;
            $use::io::copy(&mut file, &mut hasher)$(.$aw)??;

            Ok(sha1.as_ref() == &*hasher.finalize())
        }
    )*};
}
verify_impl! {
    verify_sha1(Sha1Async(Sha1::new())) = tokio @ async/await;
    verify_sha1_sync(Sha1::new()) = std;
}

struct Sha1Async(Sha1);
impl Sha1Async {
    #[inline]
    fn finalize(self) -> Output<Sha1> {
        self.0.finalize()
    }
}
impl AsyncWrite for Sha1Async {
    #[inline]
    fn poll_write(self: Pin<&mut Self>, _: &mut task::Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        Poll::Ready(self.get_mut().0.write(buf))
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(self.get_mut().0.flush())
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}


/// Wraps a [`Write`] and computes the sha1 of any written bytes
pub struct Sha1Writer<W: Write> {
    hasher: Sha1,
    writer: W,
}

impl<W: Write> Sha1Writer<W> {
    #[inline]
    pub fn new(writer: W) -> Self {
        Self {
            hasher: Sha1::new(),
            writer,
        }
    }

    #[inline]
    pub fn finalize(mut self) -> io::Result<Output<Sha1>> {
        self.writer.flush()?;
        Ok(self.hasher.finalize())
    }

    #[inline]
    pub fn finalize_str(self) -> io::Result<String> {
        Ok(format!("{:x}", self.finalize()?))
    }
}

impl<W: Write> Write for Sha1Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let count = self.writer.write(buf)?;
        self.hasher.write_all(&buf[..count]).map(|()| count)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.hasher.flush()?;
        self.writer.flush()
    }
}
