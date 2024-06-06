#![allow(missing_docs)]
//! Contains the data types used by the various operations in [Client](super::Client)

macro_rules! export {
    ($($name:ident),*$(,)?) => {$(
        mod $name;
        pub use $name::*;
    )*};
}
export! {
    project,
    version,
}

#[derive(Debug, Clone)]
pub struct Mod(pub(crate) Project);

#[derive(Debug, Clone)]
pub struct Modpack(pub(crate) Project);

macro_rules! deref {
    ($($ty:ty),*) => {$(
        impl std::ops::Deref for $ty {
            type Target = Project;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl std::ops::DerefMut for $ty {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    )*};
}
deref!(Mod, Modpack);
