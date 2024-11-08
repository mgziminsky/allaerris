use super::{Client, ClientInner};
use crate::{Error, ErrorKind};

// Need Box::pin for recursive async calls [E0733]
macro_rules! proxy {
    ($clients:expr; $name:ident($($arg:expr),*)) => {{
        let mut err = vec![];
        for client in $clients {
            let res = Box::pin(client.$name($($arg),*)).await;
            let Err(e) = res else {
                return res;
            };
            err.push(e);
        }
        Err(crate::ErrorKind::Multi(stringify!($name), err).into())
    }};
    ($clients:expr; $name:ident($($arg:expr),*) ++ $ret:ty) => {{
        let mut ret: Option<$ret> = None;
        let mut errs = vec![];
        for client in $clients {
            match Box::pin(client.$name($($arg),*)).await {
                Err(e) => errs.push(e),
                Ok(res) => {
                    use ::std::borrow::BorrowMut;
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
        // FIXME: This behavior is still sorta weird
        ret.filter(|c| errs.is_empty() || !c.is_empty())
            .ok_or_else(|| crate::ErrorKind::Multi(stringify!($name), errs).into())
    }};
}
pub(super) use proxy;


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
