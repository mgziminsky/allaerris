/// Use with [`svc_id_impl`] to set any of the clients as not available for the
/// defined type
#[allow(unused)]
pub enum Unsupported {}

/// Represents a known id for one of the supported [client](super::Client) APIs
// Prefer using [svc_id_impl] to implementing manually
#[allow(missing_docs)]
pub trait ServiceId: super::Sealed {
    type ForgeT;
    type ModrinthT;
    type GithubT;
}
macro_rules! svc_id_type {
    (@def $name:ident -> $ty:ty) => {
        svc_id_type!(@def $name -> $ty = $ty);
    };
    (@def $name:ident -> $ty:ty = $rty:ty) => {
        ::paste::paste! {
            fn [<get_ $name:lower>](&self) -> crate::Result<$rty>;
        }
    };
    (@impl $name:ident -> $ty:ty) => {
        svc_id_type!(@impl $name -> $ty = $ty);
    };
    (@impl $name:ident -> $ty:ty = $rty:ty) => {
        ::paste::paste! {
            #[inline]
            fn [<get_ $name:lower>](&self) -> crate::Result<$rty> {
                T::[<get_ $name:lower>](self)
            }
        }
    };
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            Forge($F:ty $(= $FR:ty)?),
            Modrinth($M:ty $(= $MR:ty)?),
            Github($G:ty $(= $GR:ty)?)$(,)?
        }
    ) => {
        $(#[$attr])*
        $vis enum $name {
            Forge($F),
            Modrinth($M),
            Github($G),
        }
        impl crate::client::Sealed for $name {}
        impl crate::client::ServiceId for $name {
            type ForgeT = $F;
            type ModrinthT = $M;
            type GithubT = $G;
        }
        ::paste::paste! {
            pub trait [<$name SvcType>]: Sync {
                svc_id_type!(@def Forge -> $F $(= $FR)?);
                svc_id_type!(@def Modrinth -> $M $(= $MR)?);
                svc_id_type!(@def Github -> $G $(= $GR)?);
            }

            impl<T: [<$name SvcType>] + ?Sized> [<$name SvcType>] for &T {
                svc_id_type!(@impl Forge -> $F $(= $FR)?);
                svc_id_type!(@impl Modrinth -> $M $(= $MR)?);
                svc_id_type!(@impl Github -> $G $(= $GR)?);
            }
        }
    };
}
pub(super) use svc_id_type;
