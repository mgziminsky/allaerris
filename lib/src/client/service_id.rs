use crate::Result;

pub enum Unsupported {}

pub trait ServiceId: crate::Sealed {
    type ForgeT;
    type ModrinthT;
    type GithubT;

    fn as_forge(&self) -> Result<&Self::ForgeT>;
    fn as_modrinth(&self) -> Result<&Self::ModrinthT>;
    fn as_github(&self) -> Result<&Self::GithubT>;
}
macro_rules! svc_id_impl {
    (@fn $name:ident -> Unsupported) => {
        ::paste::paste! {
            type [<$name:camel T>] = Unsupported;
            fn [<as_ $name:lower>](&self) -> $crate::Result<&Self::[<$name:camel T>]> {
                Err($crate::Error::Unsupported)
            }
        }
    };
    (@fn $name:ident -> $ty:ty) => {
        ::paste::paste! {
            type [<$name:camel T>] = $ty;
            fn [<as_ $name:lower>](&self) -> $crate::Result<&Self::[<$name:camel T>]> {
                if let Self::[<$name:camel>](v) = self {
                    Ok(v)
                } else {
                    Err($crate::Error::WrongService)
                }
            }
        }
    };

    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            Forge($F:ty),
            Modrinth($M:ty),
            Github($G:ty)$(,)?
        }
    ) => {
        $(#[$attr])*
        $vis enum $name {
            Forge($F),
            Modrinth($M),
            Github($G),
        }
        impl $crate::Sealed for $name {}
        impl $crate::client::ServiceId for $name {
            svc_id_impl!(@fn Forge -> $F);
            svc_id_impl!(@fn Modrinth -> $M);
            svc_id_impl!(@fn Github -> $G);
        }
        ::paste::paste! {
            pub type [<$name SvcType>] = dyn $crate::client::service_id::ServiceId<ForgeT = $F, ModrinthT = $M, GithubT = $G>;
        }
    };
}
pub(crate) use svc_id_impl;
