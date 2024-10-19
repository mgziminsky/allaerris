//! Library for managing Minecraft mods and modpacks from Modrinth, Curseforge,
//! and Github
#![deny(missing_docs)]

pub mod checked_types;
pub mod client;
pub mod config;
mod cow;
mod error;
mod fs_util;
mod hash;
pub mod mgmt;

use std::{env, sync::LazyLock};

// Re-export the raw clients that we wrap
mod exports {
    pub use curseforge;
    pub use github;
    pub use modrinth;
}
pub use exports::*;

use self::checked_types::PathAbsolute;
#[doc(inline)]
pub use self::{client::Client, config::Config, error::*, mgmt::ProfileManager};

/// Default directory where global config files will be stored
pub static CONF_DIR: LazyLock<PathAbsolute> = LazyLock::new(|| {
    dirs::config_local_dir()
        .expect("system config directory should be known")
        .join(env!("CARGO_PKG_NAME"))
        .try_into()
        .unwrap()
});
/// The minecraft instance directory used by the default minecraft launcher
pub static DEFAULT_MINECRAFT_DIR: LazyLock<PathAbsolute> = LazyLock::new(|| {
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

    base.expect("system home/config directory should be known")
        .join(
            #[cfg(not(target_os = "macos"))]
            ".minecraft",
            #[cfg(target_os = "macos")]
            "minecraft",
        )
        .try_into()
        .unwrap()
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
use sealed;

macro_rules! mod_export {
    ($($name:ident),*$(,)?) => {$(
        mod $name;
        pub use $name::*;
    )*};
}
use mod_export;


#[cfg(test)]
fn block_on<T>(x: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(x)
}
