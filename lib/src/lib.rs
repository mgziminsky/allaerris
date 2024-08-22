use std::{env, path::PathBuf};

use dirs::{cache_dir, config_dir, config_local_dir};
use once_cell::sync::Lazy;
use tokio::io::AsyncReadExt;

pub mod client;
pub mod config;
mod error;

// Re-export the raw clients that we wrap
pub use curseforge;
pub use error::Error;
pub use modrinth;
pub use github;

pub type Result<T> = std::result::Result<T, Error>;

pub static CONF_DIR: Lazy<PathBuf> = Lazy::new(|| config_local_dir().expect("system config directory should be known").join(env!("CARGO_PKG_NAME")));
static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| cache_dir().expect("system cache directory should be known").join(env!("CARGO_PKG_NAME")));
pub static DEFAULT_MINECRAFT_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let base = {
        #[cfg(not(target_os = "linux"))]
        {
            config_dir()
        }
        #[cfg(target_os = "linux")]
        {
            home_dir()
        }
    };

    base.expect("system home/config directory should be known").join(
        #[cfg(not(target_os = "macos"))]
        ".minecraft",
        #[cfg(target_os = "macos")]
        "minecraft",
    )
});

/// Read `source` and return the data as a string
///
/// A wrapper for dealing with the read buffer.
async fn read_wrapper(mut source: impl AsyncReadExt + Unpin) -> tokio::io::Result<String> {
    let mut buffer = String::new();
    source.read_to_string(&mut buffer).await?;
    Ok(buffer)
}

mod private {
    pub trait Sealed {}
}
use private::Sealed;
