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
    )*};
}
deref!(Mod, Modpack);
