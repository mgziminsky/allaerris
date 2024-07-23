use std::{borrow::BorrowMut, future::Future};

use super::{Client, ClientInner};
use crate::{Error, ErrorKind, Result};

/// Call `func` for each [client](Client) in order and return the first
/// successful result, or all of the errors if all calls fail
pub(super) async fn proxy<'c, Ret, F, FRet>(clients: &'c [Client], func: F) -> Result<Ret>
where
    F: Fn(&'c Client) -> FRet,
    FRet: Future<Output = Result<Ret>>,
{
    let mut err = vec![];
    for client in clients {
        // Need pin for recursive async calls [E0733]
        let res = Box::pin(func(client)).await;
        let Err(e) = res else {
            return res;
        };
        err.push(e);
    }
    Err(ErrorKind::Multi(err).into())
}

/// Call `func` for each [client](Client) in order and return their combined
/// results, or all of the errors if all calls fail
///
/// An empty result is still considered success
pub(super) async fn combined<'c, Ret, Item, F, FRet>(clients: &'c [Client], func: F) -> Result<Ret>
where
    Ret: Collection<Item>,
    F: Fn(&'c Client) -> FRet,
    FRet: Future<Output = Result<Ret>>,
{
    let mut ret: Option<Ret> = None;
    let mut errs = vec![];
    for client in clients {
        // Need pin for recursive async calls [E0733]
        match Box::pin(func(client)).await {
            Err(e) => errs.push(e),
            Ok(res) => {
                if let Some(ret) = ret.borrow_mut() {
                    // Extend results from previous success
                    ret.extend(res);
                } else {
                    // Otherwise replace error with success result
                    ret = Some(res);
                }
            },
        }
    }
    ret.filter(|c| errs.is_empty() || !c.is_empty())
        .ok_or_else(|| ErrorKind::Multi(errs).into())
}

impl TryFrom<Vec<Client>> for Client {
    type Error = Error;

    fn try_from(value: Vec<Client>) -> super::Result<Self> {
        if value.is_empty() {
            Err(ErrorKind::NoClients)?
        } else {
            Ok(ClientInner::Multi(value).into())
        }
    }
}

impl TryFrom<&Vec<Client>> for Client {
    type Error = Error;

    fn try_from(value: &Vec<Client>) -> super::Result<Self> {
        value.to_owned().try_into()
    }
}

impl TryFrom<&[Client]> for Client {
    type Error = Error;

    fn try_from(value: &[Client]) -> super::Result<Self> {
        value.to_vec().try_into()
    }
}


pub(super) trait Collection<T>: Extend<T> + IntoIterator<Item = T> {
    fn is_empty(&self) -> bool;
}
impl<T> Collection<T> for Vec<T> {
    #[inline]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }
}
impl<T: std::cmp::Ord> Collection<T> for std::collections::BTreeSet<T> {
    #[inline]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }
}
impl<T: std::hash::Hash + std::cmp::Eq> Collection<T> for std::collections::HashSet<T> {
    #[inline]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }
}
