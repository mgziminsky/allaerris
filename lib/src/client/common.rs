use std::{
    borrow::Borrow,
    cmp::Eq,
    collections::HashMap,
    future::Future,
    hash::Hash,
    path::Path,
    sync::{Arc, Mutex},
};

use async_scoped::TokioScope;
use tokio::sync::Semaphore;

use crate::Result;


/// The default latest impl since I can't figure out how to allow bodies in the
/// api macro
macro_rules! get_latest {
    () => {
        async fn get_latest(
            &self,
            id: &(impl ProjectIdSvcType + ?Sized),
            game_version: Option<&str>,
            loader: Option<ModLoader>,
        ) -> Result<Version> {
            self.get_project_versions(id, game_version, loader)
                .await?
                .into_iter()
                .max_by(|a, b| a.date.cmp(&b.date))
                .ok_or(crate::error::ErrorKind::DoesNotExist.into())
        }
    };
}
pub(super) use get_latest;

/// The default get_version impl since I can't figure out how to allow bodies in
/// the api macro
macro_rules! get_version {
    () => {
        async fn get_version(&self, id: &(impl VersionIdSvcType + ?Sized)) -> Result<Version> {
            self.get_versions(&[&id])
                .await?
                .pop()
                .ok_or(crate::error::ErrorKind::DoesNotExist.into())
        }
    };
}
pub(super) use get_version;


pub fn compute_lookup_hashes<'p, R, F, C, K>(
    files: &'p [impl AsRef<Path>],
    done: &HashMap<K, impl std::any::Any>,
    calc_fn: C,
) -> (HashMap<R, &'p Path>, Vec<crate::Error>)
where
    K: Borrow<Path> + Hash + Eq,
    R: Hash + Eq + Send + Sync,
    F: Future<Output = Result<R>> + Send + Sync,
    C: Fn(&'p Path) -> F + Send + Sync,
{
    let files = files.iter().map(AsRef::as_ref).filter(|p| !done.contains_key(*p));
    compute(files, &calc_fn).into_iter().fold((HashMap::new(), vec![]), |mut acc, res| {
        let (data, errs) = &mut acc;
        match res {
            Ok((hash, path)) => {
                data.insert(hash, path);
            },
            Err(e) => {
                errs.push(e);
            },
        }
        acc
    })
}

fn compute<'p, R, F, I, C>(files: I, calculate: &C) -> Vec<Result<(R, &'p Path)>>
where
    I: IntoIterator<Item = &'p Path>,
    R: Send + Sync,
    F: Future<Output = Result<R>> + Send + Sync,
    C: Fn(&'p Path) -> F + Send + Sync,
{
    static PERMITS: Semaphore = Semaphore::const_new(50);

    let keys = Arc::new(Mutex::new(vec![]));
    TokioScope::scope_and_block(|scope| {
        for path in files {
            let keys = keys.clone();
            scope.spawn(async move {
                let key = {
                    let _permit = PERMITS.acquire().await.unwrap();
                    calculate(path).await
                };

                keys.lock().unwrap().push(key.map(|sha1| (sha1, path)));
            });
        }
    });
    Arc::into_inner(keys).unwrap().into_inner().unwrap()
}
