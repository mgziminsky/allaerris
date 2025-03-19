use core::task::Context;
use std::{
    io::{self, Read, Write},
    path::Path,
    pin::Pin,
    task::Poll,
};

use anyhow::anyhow;
use sha1::{Digest, Sha1, digest::Output};

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

#[derive(Debug, Default)]
pub struct Sha1Async(Sha1);
impl Sha1Async {
    pub fn new() -> Self {
        Self::default()
    }

    fn finalize(self) -> Output<Sha1> {
        self.0.finalize()
    }

    pub fn finalize_str(self) -> String {
        format!("{:x}", self.0.finalize())
    }
}
impl tokio::io::AsyncWrite for Sha1Async {
    #[inline]
    fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        Poll::Ready(self.get_mut().0.write(buf))
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(self.get_mut().0.flush())
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
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


pub async fn forge_fingerprint(file: &Path) -> Result<u32> {
    use tokio::{fs, io};
    let size = match fs::metadata(file).await.map(|m| m.len()) {
        #[allow(clippy::cast_possible_truncation)]
        Ok(size) if size <= isize::MAX as _ => Some(size as _),
        Err(_) => None,
        Ok(_) => Err(anyhow!("File to large to fingerprint: `{}`", file.display()))?,
    };
    let filtered = filtered_bytes(fs::File::open(file).await.map(io::BufReader::new)?, size).await?;
    Ok(murmur2(&filtered))
}

// Forge Custom: https://github.com/AndrewToaster/ForgedCurse/blob/f44857d86db952b0931a2f9a937373a578ce90db/ForgedCurse/Utility/MurmurrHash2.cs#L68
async fn filtered_bytes(mut data: impl tokio::io::AsyncBufRead + Unpin, capacity: Option<usize>) -> Result<Vec<u8>> {
    use tokio::io::AsyncBufReadExt;
    let mut filtered = capacity.map_or_else(Vec::new, Vec::with_capacity);
    loop {
        let buf = data.fill_buf().await?;
        if buf.is_empty() {
            break;
        }
        filtered.extend(buf.iter().copied().filter(|&b| b != 0x09 && b != 0x0A && b != 0x0D && b != 0x20));
        let amt = buf.len();
        data.consume(amt);
    }
    Ok(filtered)
}

// Official Reference Impl: https://github.com/aappleby/smhasher/blob/master/src/MurmurHash2.cpp
fn murmur2(bytes: &[u8]) -> u32 {
    use std::num::Wrapping;
    const M: Wrapping<u32> = Wrapping(0x5bd1_e995);
    const R: u8 = 24;

    #[allow(clippy::cast_possible_truncation)]
    let mut hash = Wrapping(1u32 ^ (bytes.len() as u32));

    let mut chunks = bytes.chunks_exact(size_of::<u32>());
    for chunk in chunks.by_ref() {
        let mut k = Wrapping(u32::from_ne_bytes(chunk.try_into().unwrap()));
        k *= M;
        k ^= k.0 >> R;
        k *= M;

        hash *= M;
        hash ^= k;
    }
    let mut rem = chunks.remainder();
    if !rem.is_empty() {
        let mut rem_bytes = [0u8; size_of::<u32>()];
        rem.read_exact(&mut rem_bytes[..rem.len()]).unwrap();
        let rem = u32::from_ne_bytes(rem_bytes);
        hash ^= rem;
        hash *= M;
    }

    hash ^= hash >> 13;
    hash *= M;
    hash ^= hash >> 15;

    hash.0
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_on;

    #[test]
    fn murmur() {
        assert_eq!(murmur2(b"Hello World!"), 1_655_590_121);
        assert_eq!(murmur2(b"foobar"), 140_096_132);
    }

    #[test]
    fn filtered_bytes() {
        let filtered = block_on(super::filtered_bytes(&b"Hel\x09lo Wor\x0Ald\x0D!"[..], None)).unwrap();
        assert_eq!(b"HelloWorld!", filtered.as_slice());
    }

    #[test]
    fn hex_decode() {
        assert_eq!(super::hex_decode("84d8adc597166ed82e336e06ad281ada1e64bdce").unwrap().as_ref(), &[
            0x84, 0xd8, 0xad, 0xc5, 0x97, 0x16, 0x6e, 0xd8, 0x2e, 0x33, 0x6e, 0x06, 0xad, 0x28, 0x1a, 0xda, 0x1e, 0x64, 0xbd, 0xce,
        ]);
        assert_eq!(super::hex_decode("e6aaedd735ab0c3c87b4da27f32b9a2bcb0a37c1").unwrap().as_ref(), &[
            0xe6, 0xaa, 0xed, 0xd7, 0x35, 0xab, 0x0c, 0x3c, 0x87, 0xb4, 0xda, 0x27, 0xf3, 0x2b, 0x9a, 0x2b, 0xcb, 0x0a, 0x37, 0xc1,
        ]);
        assert_eq!(super::hex_decode("71120f8cdebbd0c953de70cb93439cfa0ed04f59").unwrap().as_ref(), &[
            0x71, 0x12, 0x0f, 0x8c, 0xde, 0xbb, 0xd0, 0xc9, 0x53, 0xde, 0x70, 0xcb, 0x93, 0x43, 0x9c, 0xfa, 0x0e, 0xd0, 0x4f, 0x59,
        ]);
    }

    #[test]
    fn hex_decode_invalid() {
        let Err(_) = super::hex_decode("12345") else {
            panic!("Odd length string should fail");
        };
    }
}
