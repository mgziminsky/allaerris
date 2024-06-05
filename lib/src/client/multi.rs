use std::{borrow::BorrowMut, future::Future};

use super::{Client, ClientInner};
use crate::{Error, Result};

/// Call `func` for each [client](Client) in order and return the first
/// successful result, or the first error if all calls fail
pub(super) async fn proxy<'c, Ret, F, FRet>(clients: &'c [Client], func: F) -> Result<Ret>
where
    F: Fn(&'c Client) -> FRet,
    FRet: Future<Output = Result<Ret>>,
{
    let mut err = None;
    for client in clients {
        // Need pin for recursive async calls [E0733]
        let res = Box::pin(func(client)).await;
        if res.is_ok() {
            return res;
        } else if err.is_none() {
            err = res.err();
        }
    }
    Err(err.unwrap_or(Error::DoesNotExist))
}

/// Call `func` for each [client](Client) in order and return their combined
/// results, or the first error if all calls fail.
///
/// An empty result is still considered success
pub(super) async fn combined<'c, Ret, Item, F, FRet>(clients: &'c [Client], func: F) -> Result<Ret>
where
    Ret: Extend<Item> + IntoIterator<Item = Item>,
    F: Fn(&'c Client) -> FRet,
    FRet: Future<Output = Result<Ret>>,
{
    let mut ret = None;
    for client in clients {
        // Need pin for recursive async calls [E0733]
        let res = Box::pin(func(client)).await;
        if ret.is_none() {
            // First response, keep as-is
            ret = Some(res);
        } else if let Ok(res) = res {
            // Only the first response can be an error
            // After that we only care about successes
            if let Some(Ok(ret)) = ret.borrow_mut() {
                // Extend results from previous success
                ret.extend(res);
            } else {
                // Otherwise replace error with success result
                ret = Some(Ok(res));
            }
        }
    }
    Err(err.unwrap_or(Error::DoesNotExist))
}

impl TryFrom<Vec<Client>> for Client {
    type Error = Error;

    fn try_from(value: Vec<Client>) -> super::Result<Self> {
        if value.is_empty() {
            Err(Error::NoClients)
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
