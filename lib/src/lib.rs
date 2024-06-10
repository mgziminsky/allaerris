#![warn(missing_docs)]
//! Library for managing Minecraft mods and modpacks from Modrinth, Curseforge,
//! and Github

pub mod client;
pub mod config;
mod error;

pub use std::error::Error as StdError;
use std::{env, path::PathBuf};

use once_cell::sync::Lazy;

// Re-export the raw clients that we wrap
#[rustfmt::skip]
mod exports {
    pub use curseforge;
    pub use github;
    pub use modrinth;
}
pub use exports::*;

#[doc(inline)]
pub use self::{client::Client, config::Config, error::*};

/// Default directory where global config files will be stored
pub static CONF_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::config_local_dir()
        .expect("system config directory should be known")
        .join(env!("CARGO_PKG_NAME"))
});
static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::cache_dir()
        .expect("system cache directory should be known")
        .join(env!("CARGO_PKG_NAME"))
});
/// The minecraft instance directory used by the default minecraft launcher
pub static DEFAULT_MINECRAFT_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let base = {
        #[cfg(not(target_os = "linux"))]
        {
            dirs::config_dir()
        }
        #[cfg(target_os = "linux")]
        {
            dirs::home_dir()
        }
    };

    base.expect("system home/config directory should be known").join(
        #[cfg(not(target_os = "macos"))]
        ".minecraft",
        #[cfg(target_os = "macos")]
        "minecraft",
    )
});

/// Define a local `Sealed` trait
macro_rules! sealed {
    () => {
        mod private {
            pub trait Sealed {}
        }
        use private::Sealed;
    };
}
pub(crate) use sealed;


#[cfg(test)]
pub(crate) fn block_on<T>(x: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(x)
}
