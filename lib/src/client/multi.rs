use std::future::Future;

use super::{Client, ClientInner};
use crate::{Error, Result};

pub(super) async fn proxy<'c, R, F, FR>(clients: &'c [Client], func: F) -> Result<R>
where
    F: Fn(&'c Client) -> FR,
    FR: Future<Output = Result<R>>,
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
