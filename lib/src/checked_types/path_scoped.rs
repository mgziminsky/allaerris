use std::{
    borrow::Borrow,
    ffi::{OsStr, OsString},
    ops::Deref,
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
        let path = validate_path(path.as_ref())?;
        Ok(PathScoped(path.components().collect()))
    }
}

impl Deref for PathScoped {
    type Target = PathScopedRef;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { PathScopedRef::cast(&self.0) }
    }
}
impl AsRef<PathScoped> for PathScoped {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}
impl AsRef<Path> for PathScoped {
    #[inline]
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
impl AsRef<PathScopedRef> for PathScoped {
    #[inline]
    fn as_ref(&self) -> &PathScopedRef {
        self
    }
}
impl Borrow<PathScopedRef> for PathScoped {
    #[inline]
    fn borrow(&self) -> &PathScopedRef {
        self
    }
}

impl FromStr for PathScoped {
    type Err = PathScopeError;

    #[inline]
    fn from_str(path: &str) -> Result<Self, Self::Err> {
        Self::new(path)
    }
}
impl From<PathScoped> for PathBuf {
    #[inline]
    fn from(path: PathScoped) -> Self {
        path.0
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

/// Reference version of [`PathScoped`]. Equivalent to what [`Path`] is to
/// [`PathBuf`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PathScopedRef(Path);
impl PathScopedRef {
    /// Private to ensure only created from a valid path
    #[inline]
    unsafe fn cast<P: AsRef<Path> + ?Sized>(s: &P) -> &Self {
        // Copied from Path::new
        #[allow(clippy::ref_as_ptr)]
        &*(s.as_ref() as *const Path as *const Self)
    }

    /// Creates a [`PathScopedRef`] directly referencing `path` without
    /// allocating
    ///
    /// # Errors
    ///
    /// This function will return an error if `path` is either absolute or it
    /// points at a parent scope
    pub fn new<P: AsRef<Path> + ?Sized>(path: &P) -> Result<&Self, PathScopeError> {
        let path = validate_path(path.as_ref())?;
        Ok(unsafe { Self::cast(path) })
    }

    /// Joining with another [`PathScoped`] will always produce a valid scoped
    /// path
    #[inline]
    pub fn join<P: AsRef<Self>>(&self, path: P) -> PathScoped {
        PathScoped(self.0.join(path.as_ref()))
    }

    /// Non-erroring version of [`Path::strip_prefix`] that will return `self`
    /// instead of an error when prefix does not match
    #[inline]
    pub fn remove_prefix(&self, base: impl AsRef<Path>) -> &Self {
        self.0.strip_prefix(base).map_or(self, |p| unsafe { Self::cast(p) })
    }

    /// Delegates to [`Path::parent`]
    pub fn parent(&self) -> Option<&Self> {
        self.0.parent().map(|p| unsafe { Self::cast(p) })
    }

    /// Delegates to [`Path::file_name`]
    pub fn file_name_path(&self) -> Option<&Self> {
        self.0.file_name().map(|p| unsafe { Self::cast(p) })
    }

    /// Delegates to [`Path::file_stem`]
    pub fn file_stem_path(&self) -> Option<&Self> {
        self.0.file_stem().map(|p| unsafe { Self::cast(p) })
    }

    /// Delegates to [`Path::with_file_name`]
    pub fn with_file_name<S: AsRef<OsStr>>(&self, file_name: S) -> PathScoped {
        PathScoped(self.0.with_file_name(file_name))
    }
}

impl Deref for PathScopedRef {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<PathScopedRef> for PathScopedRef {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}
impl AsRef<Path> for PathScopedRef {
    #[inline]
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
impl ToOwned for PathScopedRef {
    type Owned = PathScoped;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        PathScoped(self.0.to_path_buf())
    }
}
impl Default for &PathScopedRef {
    fn default() -> Self {
        unsafe { PathScopedRef::cast("") }
    }
}

/// Validate the path and return a reference to the portion starting at the
/// first normal component
fn validate_path(path: &Path) -> Result<&Path, PathScopeError> {
    check_scope(path)?;
    let mut comps = path.components();
    let mut clean = comps.clone();
    while let Some(std::path::Component::CurDir) = comps.next() {
        clean = comps.clone();
    }
    Ok(clean.as_path())
}
fn check_scope(path: &Path) -> Result<i32, PathScopeError> {
    use std::path::Component::*;
    path.components().try_fold(0, |depth, c| {
        let depth = depth
            + match c {
                ParentDir => -1,
                CurDir => 0,
                Normal(_) => 1,
                RootDir | Prefix(_) => return Err(PathScopeError::NonRelative),
            };
        if depth < 0 {
            Err(PathScopeError::Scoping(path.into()))
        } else {
            Ok(depth)
        }
    })
}

#[cfg(test)]
mod test_check_scope {
    use super::*;

    #[test]
    fn valid_empty() {
        let val = check_scope("".as_ref());
        assert_eq!(0, val.unwrap());
    }

    #[test]
    fn valid_normal() {
        let val = check_scope("a/b/c".as_ref());
        assert_eq!(3, val.unwrap());
    }

    #[test]
    fn valid_parent() {
        let val = check_scope("a/../c".as_ref());
        assert_eq!(1, val.unwrap());
    }

    #[test]
    fn valid_dot() {
        let val = check_scope("./a/./c".as_ref());
        assert_eq!(2, val.unwrap());
    }

    #[test]
    fn invalid_absolute() {
        let val = check_scope("/a/b/c".as_ref());
        assert!(matches!(val, Err(PathScopeError::NonRelative)), "absolute path should be an error");
    }

    #[test]
    fn invalid_parent_start() {
        let val = check_scope("../a/b/c".as_ref());
        assert!(
            matches!(val, Err(PathScopeError::Scoping(_))),
            "leading .. should produce scoping error"
        );
    }

    #[test]
    fn invalid_parent_end() {
        let val = check_scope("./a/b/../../..".as_ref());
        assert!(
            matches!(val, Err(PathScopeError::Scoping(_))),
            "more parent than subs should produce scoping error"
        );
    }
}
