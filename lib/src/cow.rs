use crate::{
    checked_types::{PathAbsolute, PathScoped, PathScopedRef},
    client::schema::{ProjectId, Version, VersionId},
};

macro_rules! cow {
    ($owned:ty) => {
        crate::cow::cow!(@ $owned, $owned);
    };
    ($owned:ty, $ref:ty) => {
        crate::cow::cow!(@{} $owned, $ref);
    };
    (@$($_:block)? $owned:ty, $ref:ty) => {
        impl<'a> From<&'a $ref> for std::borrow::Cow<'a, $ref> {
            fn from(val: &'a $ref) -> Self {
                Self::Borrowed(val)
            }
        }
        impl From<$owned> for std::borrow::Cow<'_, $ref> {
            fn from(val: $owned) -> Self {
                Self::Owned(val)
            }
        }
        impl From<std::borrow::Cow<'_, $ref>> for $owned {
            fn from(cow: std::borrow::Cow<'_, $ref>) -> Self {
                cow.into_owned()
            }
        }
        $(
            const _: () = $_;
            impl<'a> From<&'a $owned> for std::borrow::Cow<'a, $ref> {
                fn from(val: &'a $owned) -> Self {
                    Self::Borrowed(val.as_ref())
                }
            }
        )?
    };
}
pub(crate) use cow;

cow!(PathAbsolute);
cow!(PathScoped, PathScopedRef);
cow!(ProjectId);
cow!(Version);
cow!(VersionId);
