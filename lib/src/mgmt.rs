//! Profile and system management operations related to the actual downloading
//! and installing of mods and resources

mod download;
mod hash;
mod install;
mod lockfile;

pub use install::install;
