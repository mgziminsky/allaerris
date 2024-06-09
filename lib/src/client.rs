//! A unified api and schema around one or more Minecraft mod provider APIs.

mod curseforge;
mod github;
mod modrinth;
mod multi;
mod service_id;

pub mod schema;

use self::schema::{AsProjectId, Mod, Modpack, ProjectIdSvcType, Version};
pub use self::service_id::ServiceId;
use crate::{config::ModLoader, Result};

#[rustfmt::skip]
mod exported {
    /// The CurseForge API client used by this project. Convert `into` [Client](struct.Client.html) to use, don't access directly
    pub use curseforge::ApiClient as ForgeClient;
    #[doc = "The Github API client used by this project. Convert `into` [Client](struct.Client.html) to use, don't access directly\n\n"]
    pub use github::Octocrab as GithubClient;
    /// The Modrinth API client used by this project. Convert `into` [Client](struct.Client.html) to use, don't access directly
    pub use modrinth::ApiClient as ModrinthClient;
}
pub use exported::*;

crate::sealed!();

macro_rules! api {
    (@prox +) => (multi::combined);
    (@prox) => (multi::proxy);
    ($(
        $(#[$attr:meta])*
        $(+$prox:tt)?$vis:vis $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),* >)? // Generics
        ($($arg:ident: $ty:ty),*) -> $ret:ty;
    )*) => {
        trait ApiOps {$(
            async fn $name$(< $( $lt $( : $clt $(+ $dlt )* )? ),* >)?(&self, $($arg: $ty),*) -> Result<$ret>;
        )*}

        /// Methods wrapping the common api actions provided by the different services
        impl Client {$(
            $(#[$attr])*
            $vis async fn $name$(< $( $lt $( : $clt $(+ $dlt )* )? ),* >)?(&self, $($arg: $ty),*) -> Result<$ret> {
                match &self.0 {
                    ClientInner::Modrinth(c) => c.$name($($arg),*).await,
                    ClientInner::Forge(c) => c.$name($($arg),*).await,
                    ClientInner::Github(c) => c.$name($($arg),*).await,
                    ClientInner::Multi(c) => api!(@prox $($prox)?)(c, |c| c.$name($(&$arg),*)).await,
                }
            }
        )*}
    };
}
api! {
    /// Get the [mod](Mod) with `id`
    ///
    /// # Errors
    ///
    /// [[ErrorKind::InvalidIdentifier]]: if `id` fails to parse into the
    /// format expected by the backing client
    ///
    /// [[ErrorKind::WrongType]]: if the fetched project type is not a mod
    ///
    /// Any other network or api errors from the backing client
    ///
    /// [ErrorKind::InvalidIdentifier]: crate::ErrorKind::InvalidIdentifier
    /// [ErrorKind::WrongType]: crate::ErrorKind::WrongType
    pub get_mod(id: impl AsProjectId) -> Mod;

    /// Get the [modpack](Modpack) with `id`
    ///
    /// # Errors
    ///
    /// [[ErrorKind::InvalidIdentifier]]: if `id` fails to parse into the
    /// format expected by the backing client
    ///
    /// [[ErrorKind::WrongType]]: if the fetched project type is not a modpack
    ///
    /// Any other network or api errors from the backing client
    ///
    /// [ErrorKind::InvalidIdentifier]: crate::ErrorKind::InvalidIdentifier
    /// [ErrorKind::WrongType]: crate::ErrorKind::WrongType
    pub get_modpack(id: impl AsProjectId) -> Modpack;

    /// Get all [mods](Mod) listed in `ids`
    ///
    /// If called on a multi-client, then the results from all clients
    /// will be combined with no attempt to dedup. Any invalid ids will
    /// be silently ignored.
    ///
    /// # Errors
    ///
    /// Any network or api errors from the backing client
    ++pub get_mods<T: AsProjectId>(ids: impl AsRef<[T]>) -> Vec<Mod>;

    /// Get all [versions](Version) of the project with `id`
    ///
    /// If called on a multi-client, then the results from all clients
    /// will be combined with no attempt to dedup. Any invalid ids will
    /// be silently ignored.
    ///
    /// # Errors
    ///
    /// [[ErrorKind::WrongService]]: if `id` does not belong to the backing [client](Self)
    ///
    /// Any network or api errors from the backing client
    ///
    /// [ErrorKind::WrongService]: crate::ErrorKind::WrongService
    ++pub get_project_versions(id: impl AsRef<ProjectIdSvcType>, game_version: impl AsRef<Option<&str>>, loader: impl AsRef<Option<ModLoader>>) -> Vec<Version>;
}

/// The main [`Client`] for accessing the various modding APIs
///
/// Can be created using `from`/`into` with one of the supported `*Client`s
/// or a slice of them. When created from a slice of multiple clients, all
/// operations will be attempted on each client in order and the first
/// successful result will be returned. If **all** clients fail the operation,
/// then only the first [error](Error) encountered will be returned. For
/// convenience, the supported clients are re-exported as: [`ForgeClient`],
/// [`ModrinthClient`], [`GithubClient`]
///
/// # Example
/// ```no_run
/// # use relibium::client::*;
/// # async fn async_main() -> relibium::Result<()> {
/// // Single client
/// let client = Client::from(ModrinthClient::default());
/// let m = client.get_mod("mod_id").await;
/// assert!(matches!(m, Err(_)));
///
/// // Or with multiple clients
/// let client: Client = vec![
///     ModrinthClient::default().into(),
///     ForgeClient::default().into(),
///     GithubClient::builder().build()?.into(),
/// ]
/// .try_into()?;
/// let m = client.get_mod("mod_id").await;
/// assert!(matches!(m, Err(_)));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client(ClientInner);

#[derive(Debug, Clone)]
enum ClientInner {
    Modrinth(ModrinthClient),
    Forge(ForgeClient),
    Github(GithubClient),
    Multi(Vec<Client>),
}

impl From<ClientInner> for Client {
    fn from(value: ClientInner) -> Self {
        Self(value)
    }
}

macro_rules! as_inner {
    ($($ty:ty),*$(,)?) => {
        /// Methods for accessing the raw underlying service clients for
        /// performing direct queries if something isn't supported
        impl Client {
            $(::paste::paste! {
                #[doc = "Get a reference to the underlying [`"[<$ty Client>]"`] if available"]
                pub fn [<as_ $ty:lower>](&self) -> Option<&[<$ty Client>]> {
                    match &self.0 {
                        ClientInner::$ty(v) => Some(v),
                        ClientInner::Multi(clients) => clients.iter().filter_map(|c| c.[<as_ $ty:lower>]()).next(),
                        _ => None,
                    }
                }
            })*
        }
    };
}
as_inner! {
    Modrinth,
    Forge,
    Github,
}
