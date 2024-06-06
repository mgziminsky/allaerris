use std::{borrow::Borrow, cell::RefCell, path::Path};

use super::Profile;

/// Crate internal wrapper around a [profile](Profile) for use in a
/// [`Set`](std::collections) keyed on `path`
///
/// This class should only be used from config as the type inside the
/// [profiles](crate::Config::profiles) set/map
#[derive(Debug, Clone)]
pub(crate) struct ProfileByPath(RefCell<Profile>);

impl ProfileByPath {
    pub fn as_path(&self) -> &Path {
        self.borrow()
    }

    /// # UNSAFE
    /// Only call from config accessors that can guarantee this won't be aliased
    /// via guarantees on the caller
    pub fn force_mut(&self) -> &mut Profile {
        unsafe { &mut *self.0.as_ptr() }
    }
}
impl From<Profile> for ProfileByPath {
    fn from(val: Profile) -> Self {
        ProfileByPath(val.into())
    }
}
/// Required to allow set lookups by `path` only
impl Borrow<Path> for ProfileByPath {
    fn borrow(&self) -> &Path {
        unsafe { &*self.0.as_ptr() }.path()
    }
}
impl AsRef<Profile> for ProfileByPath {
    /// # UNSAFE
    /// Do not expose outside of crate
    fn as_ref(&self) -> &Profile {
        unsafe { &*self.0.as_ptr() }
    }
}
impl AsMut<Profile> for ProfileByPath {
    fn as_mut(&mut self) -> &mut Profile {
        self.0.get_mut()
    }
}


impl Eq for ProfileByPath {}
impl PartialEq for ProfileByPath {
    fn eq(&self, other: &Self) -> bool {
        self.as_path() == other.as_path()
    }
}
impl std::hash::Hash for ProfileByPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_path().hash(state)
    }
}

impl PartialOrd for ProfileByPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_path().partial_cmp(other.as_path())
    }
}
impl Ord for ProfileByPath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_path().cmp(other.as_path())
    }
}
