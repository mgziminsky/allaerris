use once_cell::sync::Lazy;

use crate::{checked_types::{PathAbsolute, PathScopedRef}, client::schema::Version};


pub static CACHE_DIR: Lazy<PathAbsolute> = Lazy::new(|| {
    dirs::cache_dir()
        .expect("system cache directory should be known")
        .join(concat!(env!("CARGO_PKG_NAME"), "-cache"))
        .try_into()
        .unwrap()
});

pub fn version_path(v: &Version, sub: Option<&PathScopedRef>) -> PathAbsolute {
    let mut path = CACHE_DIR.join(sub.unwrap_or_default());
    path.push(v.project_id.to_string());
    path.push(v.id.to_string());
    path.push(v.filename.file_name().expect("version should always contain a file name"));
    path
}
