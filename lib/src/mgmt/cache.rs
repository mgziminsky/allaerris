use std::sync::LazyLock;

use crate::{
    checked_types::{PathAbsolute, PathScopedRef},
    client::schema::{ProjectId, Version, VersionId},
};


/// The base path where files are downloaded to before they are copied into a
/// profile
pub static CACHE_DIR: LazyLock<PathAbsolute> = LazyLock::new(|| {
    dirs::cache_dir()
        .expect("system cache directory should be known")
        .join(concat!(env!("CARGO_PKG_NAME"), "-cache"))
        .try_into()
        .unwrap()
});

#[inline]
pub fn version_path(v: &Version, sub: Option<&PathScopedRef>) -> PathAbsolute {
    versioned_path(
        &v.project_id,
        &v.id,
        v.filename.file_name().expect("version should always contain a file name"),
        sub,
    )
}

pub fn versioned_path(proj_id: &ProjectId, vers_id: &VersionId, file: &std::ffi::OsStr, sub: Option<&PathScopedRef>) -> PathAbsolute {
    let mut path = CACHE_DIR.join(sub.unwrap_or_default());
    path.push(proj_id.to_string());
    path.push(vers_id.to_string());
    path.push(file);
    path
}
