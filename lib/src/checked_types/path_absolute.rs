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

    /// Just delegates to [`Path::join`] since joining on an absolute path will
    /// always produce an absolute path
    #[inline]
    pub fn join(&self, path: impl AsRef<Path>) -> Self {
        Self(self.0.join(path))
    }

    /// Consume and return the wrapped [`PathBuf`]
    #[inline]
    pub fn take(self) -> PathBuf {
        self.0
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

#[cfg(test)]
mod test {
    use std::io::Result;

    use super::*;

    #[test]
    fn valid_absolute() {
        let val: Result<PathAbsolute> = "/a/b/c".try_into();
        assert!(val.is_ok());
    }

    #[test]
    #[ignore]
    fn invalid_relative() {
        let val: Result<PathAbsolute> = "a/b/c".try_into();
        assert!(val.is_err());
    }

    #[test]
    #[ignore]
    fn invalid_parent() {
        let val: Result<PathAbsolute> = "../a/b".try_into();
        assert!(val.is_err());
    }

    #[test]
    #[ignore]
    fn invalid_dot() {
        let val: Result<PathAbsolute> = "./a/b".try_into();
        assert!(val.is_err());
    }
}