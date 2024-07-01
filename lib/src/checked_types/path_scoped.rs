use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(missing_docs)]
#[derive(Error, Debug, Clone)]
pub enum PathScopeError {
    #[error("path must be relative")]
    NonRelative,
    #[error("path must not point to an outer scope: {0}")]
    Scoping(PathBuf),
}

/// A [`PathBuf`] wrapper that is guaranteed to be relative without directly
/// referencing an outer scope. Leading [`./`] will be stripped and the path
/// partially normalized as described by [`Path::components`]
///
/// [`./`]: std::path::Component::CurDir
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "PathBuf")]
pub struct PathScoped(PathBuf);

impl PathScoped {
    /// # Errors
    ///
    /// This function will return an error if `path` is either absolute or it
    /// points at a parent scope
    pub fn new(path: impl AsRef<Path>) -> Result<Self, PathScopeError> {
        use std::path::Component::*;

        let path: &Path = path.as_ref();
        if path.has_root() {
            return Err(PathScopeError::NonRelative);
        }
        let depth_check = path.components().try_fold(0, |depth, c| {
            let depth = depth
                + match c {
                    ParentDir => -1,
                    CurDir => 0,
                    Normal(_) => 1,
                    _ => unreachable!(),
                };
            if depth < 0 {
                Err(())
            } else {
                Ok(depth)
            }
        });
        if depth_check.is_ok() {
            Ok(Self(path.components().skip_while(|c| matches!(c, CurDir)).collect()))
        } else {
            Err(PathScopeError::Scoping(path.into()))
        }
    }

    /// Joining with another [`PathScoped`] will always produce a valid scoped
    /// path
    pub fn join<P: AsRef<Self>>(&self, path: P) -> Self {
        Self(self.0.join(path.as_ref()))
    }
}

impl std::ops::Deref for PathScoped {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<PathScoped> for PathScoped {
    #[inline]
    fn as_ref(&self) -> &PathScoped {
        self
    }
}
impl AsRef<Path> for PathScoped {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl FromStr for PathScoped {
    type Err = PathScopeError;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        Self::new(path)
    }
}
macro_rules! try_from {
    ($($ty:ty),*$(,)?) => {$(
        impl TryFrom<$ty> for PathScoped {
            type Error = PathScopeError;

            #[inline]
            fn try_from(path: $ty) -> Result<Self, Self::Error> {
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

impl From<PathScoped> for PathBuf {
    fn from(path: PathScoped) -> Self {
        path.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_normal() {
        let val: Result<PathScoped, _> = "a/b/c".try_into();
        assert!(val.is_ok());
    }

    #[test]
    fn valid_parent() {
        let val: Result<PathScoped, _> = "a/../c".try_into();
        assert!(val.is_ok());
    }

    #[test]
    fn valid_dot() {
        let val: Result<PathScoped, _> = "./a/./c".try_into();
        assert!(val.is_ok());
    }

    #[test]
    fn invalid_absolute() {
        let val: Result<PathScoped, _> = "/a/b/c".try_into();
        assert!(val.is_err());
    }

    #[test]
    fn invalid_parent_start() {
        let val: Result<PathScoped, _> = "../a/b/c".try_into();
        assert!(val.is_err());
    }

    #[test]
    fn invalid_parent_end() {
        let val: Result<PathScoped, _> = "/a/b/../../..".try_into();
        assert!(val.is_err());
    }
}
