#![allow(missing_docs)]
//! Contains the data types used by the various operations in
//! [Client](super::Client)

crate::mod_export! {
    author,
    game_version,
    license,
    project,
    version,
}

macro_rules! transparent {
    {$(
        $(#[$attr:meta])*
        $vis:vis $name:ident($vis_in:vis $ty:ty)
    );*;} => {$(
        $(#[$attr])*
        $vis struct $name($vis_in $ty);

        impl core::ops::Deref for $name {
            type Target = $ty;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl core::borrow::Borrow<$ty> for $name {
            fn borrow(&self) -> &$ty {
                &self.0
            }
        }
        impl core::borrow::BorrowMut<$ty> for $name {
            fn borrow_mut(&mut self) -> &mut $ty {
                &mut self.0
            }
        }
        impl AsRef<$ty> for $name {
            fn as_ref(&self) -> &$ty {
                self
            }
        }
        impl AsMut<$ty> for $name {
            fn as_mut(&mut self) -> &mut $ty {
                self
            }
        }
    )*};
}

transparent! {
    #[derive(Debug, Clone)]
    pub Mod(pub(crate) Project);

    #[derive(Debug, Clone)]
    pub Modpack(pub(crate) Project);
}
