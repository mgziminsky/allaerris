use std::{
    borrow::Borrow,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};


/// A [`PathBuf`] wrapper that is guaranteed to be absolute.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "PathBuf")]
pub struct PathAbsolute(PathBuf);

impl PathAbsolute {
    /// See [`std::path::absolute`]
    #[inline]
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        std::path::absolute(path).map(Self)
    }

    /// Delegates to [`Path::join`] since joining on an absolute path will
    /// always produce an absolute path
    #[inline]
    #[must_use]
    pub fn join(&self, path: impl AsRef<Path>) -> Self {
        Self(self.0.join(path))
    }

    /// Consume and return the wrapped [`PathBuf`]
    #[inline]
    pub fn take(self) -> PathBuf {
        self.0
    }

    /// Delegates to [`PathBuf::push`]
    #[inline]
    pub fn push(&mut self, path: impl AsRef<Path>) {
        self.0.push(path);
    }

    /// Delegates to [`PathBuf::pop`]
    #[inline]
    pub fn pop(&mut self) -> bool {
        self.0.pop()
    }
}

impl std::ops::Deref for PathAbsolute {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<PathAbsolute> for PathAbsolute {
    #[inline]
    fn as_ref(&self) -> &PathAbsolute {
        self
    }
}
impl AsRef<Path> for PathAbsolute {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}
impl Borrow<Path> for PathAbsolute {
    #[inline]
    fn borrow(&self) -> &Path {
        self
    }
}

impl FromStr for PathAbsolute {
    type Err = std::io::Error;

    #[inline]
    fn from_str(path: &str) -> Result<Self, Self::Err> {
        Self::new(path)
    }
}
macro_rules! try_from {
    ($($ty:ty),*$(,)?) => {$(
        impl TryFrom<$ty> for PathAbsolute {
            type Error = std::io::Error;

            #[inline]
            fn try_from(path: $ty) -> std::result::Result<Self, Self::Error> {
                Self::new(path)
            }
        }
    )*};
}
try_from! {
    PathBuf,
    &Path,
    String,
    &str,
    OsString,
    &OsStr,
}

impl From<PathAbsolute> for PathBuf {
    #[inline]
    fn from(path: PathAbsolute) -> Self {
        path.take()
    }
}
