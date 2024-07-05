use std::{
    collections::{HashMap, HashSet},
    convert::identity,
    fmt::Write,
    path::Path,
};

use anyhow::{anyhow, Context};
use async_scoped::TokioScope;
use once_cell::sync::Lazy;

use super::{
    download::dl_version,
    hash::verify_sha1,
    lockfile::{LockFile, LockedId, LockedMod},
};
use crate::{
    checked_types::{PathScoped, PathScopedRef},
    client::schema::{ProjectId, Version, VersionId},
    config::{profile::ProfileData, Profile},
    Client, ErrorKind, Result,
};

static MODS_PATH: Lazy<&PathScopedRef> = Lazy::new(|| PathScopedRef::new("mods").unwrap());

/// Attempt to download and install all mods defined in [`profile`](Profile).
///
/// Any previously installed mods that were removed from the profile will be
/// deleted. Updated versions of mods will delete the old version after
/// installing the update.
///
/// Will return a tuple containing the list of all successfully installed mods
/// and a list of installation errors. If the error list is not empty, it is
/// possible the profile could be in an incomplete state
///
/// # Errors
///
/// This function will only return an error if a lock file exists and fails to
/// parse. Otherwise, a list of individual install errors will be returned with
/// the successful result
pub async fn install(client: &Client, profile: &Profile) -> Result<(Vec<LockedMod>, Vec<crate::Error>)> {
    let data = profile.data().await?;

    let mut lockfile = LockFile::load(profile.path()).await?;
    let reset = lockfile.game_version != data.game_version || lockfile.loader != data.loader;
    lockfile.game_version = data.game_version.clone();
    lockfile.loader = data.loader;

    let (mut versioned, mut unversioned) = profile_mods(data);
    // TODO: Load mod list from modpack
    let (mut installed, locked_versions, delete) =
        merge_locked(lockfile.mods, &mut versioned, &mut unversioned, profile.path(), reset).await;

    let (pending, mut errors) = fetch_versions(client, data, unversioned, &locked_versions, &versioned).await?;

    let mut unversioned = versioned.into_keys().chain(locked_versions.keys()).collect::<HashSet<_>>();
    for v in pending.iter() {
        unversioned.remove(&v.project_id);
    }
    for id in unversioned {
        errors.push(ErrorKind::MissingVersion(id.clone()).into());
    }

    let ((), downloads) = TokioScope::scope_and_block(|scope| {
        for v in pending {
            scope.spawn(dl_version(v));
        }
    });
    for dl in downloads {
        match dl {
            Ok(Ok((v, cached))) => match install_version(profile.path(), v, &cached).await {
                Ok(lm) => installed.push(lm),
                Err(e) => errors.push(e),
            },
            Ok(Err(e)) => errors.push(e),
            Err(e) => errors.push(anyhow!(e).into()),
        }
    }

    let delete_job = spawn_delete_files(&delete, profile.path());

    lockfile.mods = installed;
    lockfile.mods.sort_unstable_by(|a, b| a.file.cmp(&b.file));
    if let Err(e) = lockfile.save(profile.path()).await {
        errors.push(e);
    };

    if let Ok(errs) = delete_job.await {
        errors.extend(errs)
    } else {
        let mut files = String::with_capacity(2 * delete.len() + delete.iter().map(|p| p.as_os_str().len()).sum::<usize>());
        for file in delete {
            files.push_str("\n\t");
            let _ = write!(&mut files, "{}", file.display());
        }
        errors.push(anyhow!("Unexpected error deleting old mods. The following files may need deleted manually:{files}",).into())
    }

    Ok((lockfile.mods, errors))
}


fn profile_mods(data: &ProfileData) -> (HashMap<&ProjectId, &VersionId>, HashSet<&ProjectId>) {
    let mut versioned = HashMap::new();
    let mut unversioned = HashSet::new();
    for m in &data.mods {
        if let Some(v) = m.id.version() {
            versioned.insert(m.id.project(), v);
        } else {
            unversioned.insert(m.id.project());
        }
    }
    (versioned, unversioned)
}

async fn merge_locked(
    locked_mods: Vec<LockedMod>,
    versioned: &mut HashMap<&ProjectId, &VersionId>,
    unversioned: &mut HashSet<&ProjectId>,
    profile_path: &Path,
    reset: bool,
) -> (Vec<LockedMod>, HashMap<ProjectId, VersionId>, Vec<PathScoped>) {
    let mut installed = Vec::with_capacity(versioned.len() + unversioned.len());
    let mut locked_versions = HashMap::new();
    let mut delete = vec![];
    for m in locked_mods {
        // If locked mod isn't in profile or it has a different version, delete it
        if (reset || !unversioned.remove(&m.id.project)) && versioned.get(&m.id.project).into_iter().all(|pv| **pv != m.id.version) {
            delete.push(m.file);
            continue;
        }
        let path = profile_path.join(&m.file);
        if path.exists() && verify_sha1(&m.sha1, &path).await.is_ok_and(identity) {
            versioned.remove(&m.id.project);
            installed.push(m);
        } else {
            locked_versions.insert(m.id.project, m.id.version);
        }
    }
    (installed, locked_versions, delete)
}

/// Fetch the [version] details of all mods that will be installed
///
/// [version]: crate::client::schema::Version
async fn fetch_versions(
    client: &Client,
    data: &ProfileData,
    unversioned: HashSet<&ProjectId>,
    locked_versions: &HashMap<ProjectId, VersionId>,
    versioned: &HashMap<&ProjectId, &VersionId>,
) -> Result<(Vec<Version>, Vec<crate::Error>)> {
    // Get the latest version of all unversioned projects
    let ((), pending) = TokioScope::scope_and_block(|scope| {
        for id in unversioned {
            scope.spawn(client.get_latest(id, Some(&data.game_version), Some(data.loader)));
        }
    });
    let (mut pending, mut errors) = pending.into_iter().fold((vec![], vec![]), |mut acc, res| {
        let (versions, errors) = &mut acc;
        match res {
            Ok(Ok(v)) => versions.push(v),
            Ok(Err(e)) => errors.push(e),
            Err(e) => errors.push(anyhow!(e).into()),
        }
        acc
    });

    // Fetch version details for all versioned projects
    let versions = client
        .get_versions(
            &locked_versions
                .values()
                .chain(versioned.values().copied())
                .map(|v| v as _)
                .collect::<Vec<_>>(),
        )
        .await;
    match versions {
        Ok(versions) => pending.extend(versions),
        Err(e) => errors.push(e),
    }

    Ok((pending, errors))
}

async fn install_version(profile_path: &Path, v: Version, cached: &Path) -> Result<LockedMod> {
    let lm = LockedMod {
        id: LockedId::new(v.project_id, v.id)?,
        file: MODS_PATH.join(v.filename.remove_prefix(*MODS_PATH)),
        sha1: v.sha1.expect("downloaded mod should always have a sha1"),
    };
    let dest = profile_path.join(&lm.file);
    let _ = tokio::fs::create_dir_all(dest.parent().expect("dest directory should always be valid")).await;
    tokio::fs::copy(cached, dest)
        .await
        .context("Failed to copy downloaded mod into profile")?;
    Ok(lm)
}

#[must_use]
fn spawn_delete_files(delete: &[PathScoped], profile_path: &Path) -> tokio::task::JoinHandle<Vec<crate::Error>> {
    let delete = delete.iter().map(|p| profile_path.join(p)).collect::<Vec<_>>();
    tokio::task::spawn_blocking(|| {
        delete
            .into_iter()
            .filter_map(|file| std::fs::remove_file(file).err())
            .map(Into::into)
            .collect::<Vec<_>>()
    })
}
