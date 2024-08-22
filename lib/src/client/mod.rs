//! A unified api and schema around one or more Minecraft mod provider APIs.

mod curseforge;
mod github;
mod modrinth;
mod multi;
mod service_id;

pub mod schema;

use schema::{Mod, Modpack, ProjectIdSvcType, Version};
pub use service_id::*;

use crate::{config::ModLoader, Result};

#[rustfmt::skip]
mod exported {
    pub use modrinth::ApiClient as RawModrinthClient;
    pub use github::Octocrab as RawGithubClient;
    pub use curseforge::ApiClient as RawForgeClient;
}
pub use exported::*;

macro_rules! api {
    ($(
        $(#[$attr:meta])*
        $vis:vis $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;
    )*) => {
        trait ApiOps {$(
            async fn $name(&self, $($arg: $ty),*) -> Result<$ret>;
        )*}

        impl Client {$(
            $(#[$attr])*
            $vis async fn $name(&self, $($arg: $ty),*) -> Result<$ret> {
                match &self.0 {
                    ClientInner::Modrinth(c) => c.$name($($arg),*).await,
                    ClientInner::Forge(c) => c.$name($($arg),*).await,
                    ClientInner::Github(c) => c.$name($($arg),*).await,
                    ClientInner::Multi(c) => multi::proxy(c, |c| c.$name($(&$arg),*)).await,
                }
            }
        )*}
    };
}
api! {
    pub get_mod(id: impl AsRef<str>) -> Mod;
    pub get_modpack(id: impl AsRef<str>) -> Modpack;
    pub get_mods(ids: impl AsRef<[&str]>) -> Vec<Mod>;
    pub get_project_versions(id: impl AsRef<ProjectIdSvcType>, game_version: impl AsRef<Option<&str>>, loader: impl AsRef<Option<ModLoader>>) -> Vec<Version>;
}

/// The main [`Client`] for accessing the various modding APIs
///
/// Can be created using `from`/`into` with one of the supported `Raw*Client`s or a slice of them.
/// When created from a slice of multiple clients, all operations will be attempted on each
/// client in order and the first successful result will be returned. If *all* clients fail
/// the operation, then only the first error encountered will be returned.
/// For convenience, the supported clients are re-exported as: [`RawForgeClient`], [`RawModrinthClient`],
/// [`RawGithubClient`]
///
/// # Example
/// ```no_run
/// // Single client
/// let client = Client::from(RawModrinthClient::default());
/// let m = client.get_mod("mod_id").await;
/// assert!(matches!(m, Err(_)));
///
/// // Or with multiple clients
/// let client: Client = vec![
///     RawModrinthClient::default().into(),
///     RawForgeClient::default().into(),
///     RawGithubClient::builder().build()?.into(),
/// ]
/// .try_into()?;
/// let m = client.get_mod("mod_id").await;
/// assert!(matches!(m, Err(_)));
/// ```
#[derive(Debug, Clone)]
pub struct Client(ClientInner);

#[derive(Debug, Clone)]
enum ClientInner {
    Modrinth(RawModrinthClient),
    Forge(RawForgeClient),
    Github(RawGithubClient),
    Multi(Vec<Client>),
}

impl From<ClientInner> for Client {
    fn from(value: ClientInner) -> Self {
        Self(value)
    }
}
