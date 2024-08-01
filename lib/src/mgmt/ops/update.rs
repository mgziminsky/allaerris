use std::collections::HashMap;

use crate::{
    client::schema::ProjectId,
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


#[allow(clippy::missing_panics_doc)] // Map indexing should be safe
impl ProfileManager {
    /// Updates any installed profile mods without an explicit version to their
    /// latest compatible version
    pub async fn update(&self, client: &Client, profile: &Profile) -> Result<()> {
        let profile_path = profile.path();
        let mut lockfile = LockFile::load(profile_path).await?;
        if lockfile.mods.is_empty() && lockfile.pack.is_none() {
            return Ok(());
        }

        let data = profile.data().await?;
        let pending = get_updatable(data, lockfile.pack.as_ref(), &lockfile.mods);
        if pending.is_empty() {
            return Ok(());
        }

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
            lockfile.outdated.push(core::mem::replace(lm, ulm));
        }

        lockfile.sort();
        lockfile.save(profile_path).await?;

        Ok(())
    }

    /// Cancel the update of any outdated mods waiting to be installed
    pub async fn revert(&self, profile: &Profile) -> Result<()> {
        let profile_path = profile.path();
        let mut lockfile = LockFile::load(profile_path).await?;
        if lockfile.outdated.is_empty() {
            return Ok(());
        }

        let mods = get_updatable(profile.data().await?, lockfile.pack.as_ref(), &lockfile.mods);
        for prev in lockfile.outdated.drain(..) {
            let lm = get_mod!(lockfile, mods[prev.project()], mut);
            *lm = prev;
        }

        lockfile.sort();
        lockfile.save(profile_path).await?;

        Ok(())
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
