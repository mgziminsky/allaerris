use std::collections::HashMap;

use crate::{
    checked_types::PathScoped,
    client::schema::{ProjectId, VersionId},
    config::{profile::ProfileData, Profile, VersionedProject},
    mgmt::lockfile::{LockFile, LockedMod, LockedPack},
    Client, ProfileManager, Result,
};


macro_rules! get_mod {
    ($l:expr, $i:expr $(, $mut:ident)?) => {
        paste::paste! {
            $l.mods.[<get $(_ $mut)?>]($i).unwrap_or_else(|| $l.pack.[<as_deref $(_ $mut)?>]().unwrap())
        }
    };
}

#[allow(missing_docs)] // fields are self explanatory
/// Basic version info for an updated project
pub struct UpdateInfo {
    pub project: ProjectId,
    pub from: (VersionId, PathScoped),
    pub to: (VersionId, PathScoped),
}

#[allow(clippy::missing_panics_doc)] // Map indexing should be safe
impl ProfileManager {
    /// Updates any installed profile mods without an explicit version to their
    /// latest compatible version
    pub async fn update(&self, client: &Client, profile: &Profile) -> Result<Vec<UpdateInfo>> {
        let profile_path = profile.path();
        let mut lockfile = LockFile::load(profile_path).await?;
        if lockfile.mods.is_empty() && lockfile.pack.is_none() {
            return Ok(vec![]);
        }

        let data = profile.data().await?;
        let pending = get_updatable(data, lockfile.pack.as_ref(), &lockfile.mods);
        if pending.is_empty() {
            return Ok(vec![]);
        }

        let mut updated = vec![];
        let updates = client
            .get_updates(
                &data.game_version,
                data.loader,
                &pending.values().map(|&i| get_mod!(lockfile, i)).collect::<Vec<_>>(),
            )
            .await?
            .into_iter()
            .map(|lm| (pending[lm.project()], lm))
            .collect::<Vec<_>>();
        lockfile.outdated.reserve(updates.len());

        for (i, mut ulm) in updates {
            let lm = get_mod!(lockfile, i, mut);

            // Keep the current subdir if update doesn't specify
            match ulm.file.parent() {
                Some(p) if p.as_os_str().is_empty() => {
                    ulm.file = lm.file.with_file_name(ulm.file.as_os_str());
                },
                _ => {},
            }
            updated.push(UpdateInfo {
                project: lm.project().clone(),
                from: (lm.id.version.clone(), lm.file.clone()),
                to: (ulm.id.version.clone(), ulm.file.clone()),
            });
            lockfile.outdated.push(core::mem::replace(lm, ulm));
        }

        lockfile.sort();
        lockfile.save(profile_path).await?;

        Ok(updated)
    }

    /// Cancel the update of any outdated mods waiting to be installed
    pub async fn revert(&self, profile: &Profile) -> Result<Vec<UpdateInfo>> {
        let profile_path = profile.path();
        let mut lockfile = LockFile::load(profile_path).await?;
        if lockfile.outdated.is_empty() {
            return Ok(vec![]);
        }

        let mut updated = vec![];
        let mods = get_updatable(profile.data().await?, lockfile.pack.as_ref(), &lockfile.mods);
        for prev in lockfile.outdated.drain(..) {
            let lm = get_mod!(lockfile, mods[prev.project()], mut);
            updated.push(UpdateInfo {
                project: lm.project().clone(),
                from: (lm.id.version.clone(), lm.file.clone()),
                to: (prev.id.version.clone(), prev.file.clone()),
            });
            *lm = prev;
        }

        lockfile.sort();
        lockfile.save(profile_path).await?;

        Ok(updated)
    }
}

/// Get the unversioned profile project ids that can be updated
///
/// [`ProjectId`]`-> mods index` where the pack index = `mods.len()`
fn get_updatable<'p, 'l>(data: &'p ProfileData, pack: Option<&'l LockedPack>, mods: &'l [LockedMod]) -> HashMap<&'p ProjectId, usize> {
    let mut pending = data
        .mods
        .iter()
        .filter_map(|m| m.version().is_none().then_some((m.project(), None)))
        .collect::<HashMap<_, _>>();
    if let Some(pack) = data.modpack.as_ref().filter(|p| p.version().is_none()) {
        pending.insert(pack.project(), None);
    }

    // Set values to installed projects
    if let Some(pack) = pack {
        if let Some(v) = pending.get_mut(pack.project()) {
            v.replace(mods.len());
        }
    }
    for (i, m) in mods.iter().enumerate() {
        if let Some(v) = pending.get_mut(m.project()) {
            *v = Some(i);
        }
    }

    // Remove any projects that aren't installed and unwrap
    pending.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect()
}
