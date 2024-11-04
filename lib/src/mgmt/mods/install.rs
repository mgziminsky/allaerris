use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
    convert::identity,
    mem::take,
    ops::Deref,
    path::Path,
    sync::{Arc, LazyLock},
};

use anyhow::{anyhow, Context};
use async_scoped::TokioScope;
use itertools::Itertools;

use crate::{
    checked_types::{PathAbsolute, PathScoped, PathScopedRef},
    client::schema::{ProjectId, Version, VersionId},
    config::{profile::ProfileData, Mod, Profile, VersionedProject},
    hash::{verify_sha1, verify_sha1_sync, Sha1Writer},
    mgmt::{
        cache,
        events::{EventSouce, InstallType, ProgressEvent},
        lockfile::{LockFile, LockedMod, LockedPack, PathHashes},
        modpack::{modrinth::IndexFile, ModpackData},
        version::VersionSet,
        ProfileManager,
    },
    Client, ErrorKind, Result, StdResult,
};

type Downloads = Vec<StdResult<Option<(Version, PathAbsolute)>, tokio::task::JoinError>>;

static MODS_PATH: LazyLock<&PathScopedRef> = LazyLock::new(|| PathScopedRef::new("mods").unwrap());


// Public interface
impl ProfileManager {
    /// Attempt to download and install all mods defined in
    /// [`profile`](Profile).
    ///
    /// Any previously installed mods that were removed from the profile will be
    /// deleted. Updated versions of mods will delete the old version after
    /// installing the update.
    ///
    /// Progress and most errors will be sent to [`channel`]
    ///
    /// # Errors
    ///
    /// This function will only return an error if a lock file exists and fails
    /// to parse. Otherwise, any other individual errors will be sent to the
    /// [`channel`]
    ///
    /// [`channel`]: Self::with_channel
    pub async fn apply(&self, client: &Client, profile: &Profile) -> Result<()> {
        let data = profile.data().await?;
        let profile_path = profile.path();

        self.send(ProgressEvent::Status("Loading lockfile...".to_string()));
        let mut lockfile = LockFile::load(profile_path).await?;

        self.install(client, profile_path, data, &mut lockfile).await?;

        lockfile.game_version.clone_from(&data.game_version);
        lockfile.loader = data.loader;
        lockfile.sort();
        if let Err(e) = lockfile.save(profile_path).await {
            self.send_err(e);
        };

        Ok(())
    }
}

// Internal helpers
impl ProfileManager {
    async fn install(&self, client: &Client, profile_path: &PathAbsolute, data: &ProfileData, lockfile: &mut LockFile) -> Result<()> {
        let mut delete = take(&mut lockfile.outdated).into_iter().map(|lm| lm.file).collect();

        let mut pack = self.load_pack(client, profile_path, data, lockfile, &mut delete).await?;

        self.send(ProgressEvent::Status("Resolving mod versions...".to_string()));
        let reset = lockfile.game_version != data.game_version || lockfile.loader != data.loader;
        let ResolvedMods {
            versioned,
            unversioned,
            installed,
            mut pending,
        } = merge_sources(
            &data.mods,
            &lockfile.mods,
            pack.as_mut(),
            profile_path,
            &mut delete,
            reset,
            self.force,
        )
        .await;

        self.fetch_versions(client, data, unversioned, versioned, &mut pending).await;
        let downloads = self.download_files(pending, profile_path);

        self.send(ProgressEvent::Status("Installing...".to_string()));
        lockfile.mods = installed
            .into_iter()
            .inspect(|m| {
                self.send(ProgressEvent::Installed {
                    file: m.file.clone(),
                    is_new: false,
                    typ: InstallType::Mod,
                });
            })
            .map(Cow::into_owned)
            .collect();
        lockfile.mods.extend(self.install_downloaded(downloads, profile_path).await);

        if let Some(pack) = pack {
            delete.extend(self.install_pack(pack, lockfile, profile_path, data).into_keys());
            // Don't delete extracted overrides
            for p in lockfile.pack.as_ref().unwrap().overrides.keys() {
                delete.remove(p);
            }
        }

        // Don't delete anything that was just installed
        for p in lockfile.mods.iter().map(|m| &m.file).chain(lockfile.other.keys()) {
            delete.remove(p);
        }
        if self.delete_files(delete.iter(), profile_path).await.is_err() {
            let files = delete.iter().map(|p| p.display()).join("\n\t");
            self.send_err(anyhow!("Unexpected error deleting old files. The following may need deleted manually:\n\t{files}").into());
        }

        Ok(())
    }

    fn install_pack(&self, pack: ModpackData, lockfile: &mut LockFile, profile_path: &PathAbsolute, data: &ProfileData) -> PathHashes {
        use crate::mgmt::modpack::PackMods::Modrinth;

        let mut delete = PathHashes::new();
        if let Modrinth { ref unknown, .. } = pack.mods {
            delete.extend(self.install_modrinth_unknown(&mut lockfile.other, profile_path, unknown));
        }
        if data.modpack.as_ref().is_some_and(|mp| mp.install_overrides) {
            let locked_pack = lockfile.pack.as_mut().expect("Should have lockfile pack if we have pack data");
            let mut removed = self.extract_overrides(&mut locked_pack.overrides, pack, profile_path);
            // Don't delete any overrides that have been modified
            removed.retain(|path, sha1| {
                let unchanged = verify_sha1_sync(sha1, &profile_path.join(path)).unwrap_or(true);
                if !unchanged {
                    self.send(ProgressEvent::Status(format!(
                        "Keeping removed override due to modifications: {}",
                        path.display()
                    )));
                }
                unchanged
            });
            delete.extend(removed);
        }
        delete
    }

    /// Fetches and loads the modpack data from the pack index.
    async fn load_pack(
        &self,
        client: &Client,
        profile_path: &PathAbsolute,
        data: &ProfileData,
        lockfile: &mut LockFile,
        delete: &mut BTreeSet<PathScoped>,
    ) -> Result<Option<ModpackData>> {
        macro_rules! fetch_replace {
            ($pack:expr) => {
                self.fetch_pack(client, $pack.deref(), data).await.map(|(data, lm)| {
                    lockfile.pack.replace(LockedPack::new(lm));
                    Some(data)
                })
            };
        }
        // Marks unchanged overrides for deletion
        macro_rules! delete_overrides {
            ($lp:expr) => {
                for (p, s) in take(&mut $lp.overrides) {
                    if let Ok(true) = verify_sha1(&s, &profile_path.join(&p)).await {
                        delete.insert(p);
                    } else {
                        self.send(ProgressEvent::Status(format!(
                            "Keeping changed override file: {}",
                            p.display()
                        )));
                    }
                }
            };
        }
        self.send(ProgressEvent::Status("Fetch and read modpack...".to_string()));
        match (lockfile.pack.as_mut(), &data.modpack) {
            (None, None) => Ok(None),
            (Some(lp), None) => {
                delete_overrides!(lp);
                Ok(None)
            },
            (Some(lp), Some(pack)) => {
                if pack.version().is_some_and(|v| v != &lp.id.version) {
                    delete_overrides!(lp);
                    fetch_replace!(pack)
                } else {
                    if !pack.install_overrides {
                        delete_overrides!(lp);
                    }
                    let cached = cache::versioned_path(
                        &lp.id.project,
                        &lp.id.version,
                        lp.file.as_os_str(),
                        PathScopedRef::new("modpacks").ok(),
                    );
                    if !self.force && cached.exists() && verify_sha1(&lp.sha1, &cached).await.is_ok_and(identity) {
                        self.read_pack(client, &cached).await
                    } else {
                        self.fetch_pack(client, lp, data).await.map(|(data, _)| data)
                    }
                    .map(Some)
                }
            },
            (None, Some(pack)) => fetch_replace!(pack),
        }
    }

    async fn fetch_pack(&self, client: &Client, pack: &impl VersionedProject, data: &ProfileData) -> Result<(ModpackData, LockedMod)> {
        let (v, data) = self.load_modpack(client, pack, data).await?;
        Ok((data, v.into()))
    }

    /// Fetch the [version] details of all mods that will be installed
    ///
    /// [version]: crate::client::schema::Version
    async fn fetch_versions(
        &self,
        client: &Client,
        data: &ProfileData,
        unversioned: HashSet<Cow<'_, ProjectId>>,
        versioned: HashMap<Cow<'_, ProjectId>, Cow<'_, VersionId>>,
        out_pending: &mut VersionSet,
    ) {
        self.send(ProgressEvent::Status("Fetch version details...".to_string()));
        // Get the latest version of all unversioned projects
        let ((), pending) = TokioScope::scope_and_block(|scope| {
            let semaphore = Arc::new(tokio::sync::Semaphore::const_new(10));
            for id in &unversioned {
                let semaphore = semaphore.clone();
                scope.spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    client.get_latest(id.as_ref(), Some(&data.game_version), data.loader.known()).await
                });
            }
        });
        for res in pending {
            match res {
                Ok(Ok(v)) => {
                    out_pending.replace(v.into());
                },
                Ok(Err(e)) => self.send_err(e),
                Err(e) => self.send_err(anyhow!(e).into()),
            }
        }

        // Fetch version details for all versioned projects
        match client
            .get_versions(&versioned.values().map(|v| v.as_ref() as _).collect::<Vec<_>>())
            .await
        {
            Ok(versions) => {
                for v in versions {
                    out_pending.replace(v.into());
                }
            },
            Err(e) => self.send_err(e),
        }

        // Notify of any unknown projects
        let mut unversioned = versioned.into_keys().chain(unversioned).collect::<HashSet<_>>();
        for v in out_pending.iter() {
            unversioned.remove(&v.project_id);
        }
        for id in unversioned {
            self.send_err(ErrorKind::MissingVersion(id.into_owned()).into());
        }
    }

    fn download_files(&self, pending: VersionSet, profile_path: &PathAbsolute) -> Downloads {
        if !pending.is_empty() {
            self.send(ProgressEvent::Status("Downloading...".to_string()));
        }
        let ((), downloads) = TokioScope::scope_and_block(|scope| {
            for v in pending {
                let sub = v.filename.parent().and_then(|p| p.file_name_path()).unwrap_or(&MODS_PATH);
                let save_path = if self.no_cache {
                    profile_path.join(sub)
                } else {
                    cache::version_path(&v, Some(sub))
                };
                scope.spawn(async move {
                    self.download(&*v, &save_path).await.map(|sha1| {
                        let mut v = v.into_inner();
                        v.sha1.replace(sha1);
                        (v, save_path)
                    })
                });
            }
        });
        downloads
    }

    #[must_use]
    fn delete_files<'a>(&self, delete: impl Iterator<Item = &'a PathScoped>, profile_path: &PathAbsolute) -> tokio::task::JoinHandle<()> {
        let delete = delete
            .filter_map(|p| {
                let file = profile_path.join(p);
                file.exists().then_some((p.to_owned(), file))
            })
            .collect::<Vec<_>>();
        if !delete.is_empty() {
            self.send(ProgressEvent::Status("Deleting removed/outdated files...".into()));
        }
        let channel = self.channel.clone();
        tokio::task::spawn_blocking(move || {
            for (p, file) in delete {
                match std::fs::remove_file(file).with_context(|| format!("Failed to delete file `{}`", p.display())) {
                    Ok(()) => channel.send(ProgressEvent::Deleted(p)),
                    Err(e) => channel.send(ProgressEvent::Error(e.into())),
                }
            }
        })
    }

    async fn install_downloaded(&self, downloads: Downloads, profile_path: &PathAbsolute) -> Vec<LockedMod> {
        let mut installed = Vec::with_capacity(downloads.len());
        for dl in downloads {
            match dl {
                Ok(Some((v, file_path))) => match install_version(profile_path, v, &file_path).await {
                    Ok(lm) => {
                        self.send(ProgressEvent::Installed {
                            file: lm.file.clone(),
                            is_new: true,
                            typ: InstallType::Mod,
                        });
                        installed.push(lm);
                    },
                    Err(e) => self.send_err(e),
                },
                Ok(None) => { /* Download failed. Error already sent to channel */ },
                Err(e) => self.send_err(anyhow!(e).into()),
            }
        }
        installed
    }

    /// Install any additional files from a Modrinth pack that weren't
    /// recognized as a project on one of the supported APIs.
    ///
    /// Returns any previously installed files that are no longer present in the
    /// pack
    fn install_modrinth_unknown(&self, lock: &mut PathHashes, profile_path: &PathAbsolute, unknown: &[IndexFile]) -> PathHashes {
        let mut to_delete = take(lock);
        if unknown.is_empty() {
            return to_delete;
        }

        self.send(ProgressEvent::Status("Installing remaining external pack mods...".into()));
        let ((), downloads) = TokioScope::scope_and_block(|scope| {
            for file in unknown {
                let Ok(path) = file.path_scoped() else {
                    continue;
                };
                to_delete.remove(path);
                scope.spawn(async move {
                    let target = &profile_path.join(path);
                    let path = path.to_owned();
                    if !self.force && verify_sha1(&file.hashes.sha1, target).await.unwrap_or(false) {
                        Some((file.hashes.sha1.clone(), path, false))
                    } else {
                        self.download(file, target).await.map(|sha1| (sha1, path, true))
                    }
                });
            }
        });
        for dl in downloads {
            match dl {
                Ok(Some((sha1, file, is_new))) => {
                    lock.insert(file.clone(), sha1);
                    self.send(ProgressEvent::Installed {
                        file,
                        is_new,
                        typ: InstallType::Other,
                    });
                },
                Ok(None) => { /* Download failed. Error already sent to channel */ },
                Err(e) => self.send_err(anyhow!(e).into()),
            }
        }
        to_delete
    }

    /// Extracts override files from the pack. Any previosly installed overrides
    /// that changed since install will be backed up before overwriting.
    ///
    /// Returns any previously installed override files that are no longer
    /// present in the pack
    fn extract_overrides(&self, overrides: &mut PathHashes, mut pack: ModpackData, profile_path: &PathAbsolute) -> PathHashes {
        self.send(ProgressEvent::Status("Extracting Overrides...".to_string()));
        let mut to_delete = take(overrides);
        pack.visit_overrides(|path, mut file| {
            use std::{fs, io};

            let target = &profile_path.join(path);
            if let Some(sha1) = to_delete.remove(path) {
                if let Ok(false) = verify_sha1_sync(&sha1, target) {
                    let bak = {
                        let mut bak = target.to_path_buf();
                        bak.as_mut_os_string().push(".bak");
                        bak
                    };
                    if fs::rename(target, &bak).is_ok() {
                        self.send(ProgressEvent::Status(format!(
                            "Created backup of modified override file: {}",
                            bak.display()
                        )));
                    } else {
                        self.send_err(
                            anyhow!(
                                "Failed to create backup of modified override file. It will not be extracted\n\t{}",
                                path.display()
                            )
                            .into(),
                        );
                        return;
                    }
                };
            }

            target.parent().map(fs::create_dir_all);
            let sha1 = fs::File::create(target)
                .map(io::BufWriter::new)
                .map(Sha1Writer::new)
                .and_then(|mut target| {
                    io::copy(&mut file, &mut target)?;
                    target.finalize_str()
                });
            match sha1 {
                Ok(sha1) => {
                    overrides.insert(path.to_owned(), sha1);
                    self.send(ProgressEvent::Installed {
                        file: path.to_owned(),
                        is_new: true,
                        typ: InstallType::Override,
                    });
                },
                Err(e) => self.send_err(e.into()),
            }
        });
        to_delete
    }
}

async fn merge_sources<'a>(
    profile: &'a Vec<Mod>,
    locked: &'a Vec<LockedMod>,
    pack: Option<&'a mut ModpackData>,
    profile_path: &'a Path,
    delete: &mut BTreeSet<PathScoped>,
    reset: bool,
    force: bool,
) -> ResolvedMods<'a> {
    let mut resolved = ResolvedMods {
        installed: Vec::with_capacity(locked.len()),
        ..Default::default()
    };
    let ResolvedMods {
        versioned,
        unversioned,
        installed,
        pending,
    } = &mut resolved;

    // Modpack first as base set of mods
    {
        use crate::mgmt::modpack::PackMods::*;
        match pack.map(|p| &mut p.mods) {
            Some(Modrinth { known, .. }) => std::mem::swap(pending, known),
            Some(Forge(mods)) => std::mem::swap(versioned, &mut mods.iter().map(|(k, v)| (k.into(), v.into())).collect()),
            None => {},
        }
    }
    // Then mods from the profile
    for m in profile {
        let pid = m.project();
        if m.exclude {
            pending.remove(pid);
            continue;
        }
        match (pending.get(pid), m.version()) {
            (None, None) => {
                unversioned.insert(pid.into());
            },
            (None, Some(v)) => {
                versioned.insert(pid.into(), v.into());
            },
            (Some(mpv), Some(v)) if mpv.id != v => {
                versioned.insert(pid.into(), v.into());
                pending.remove(pid);
            },
            (Some(_), Some(_) | None) => { /* Keep modpack version */ },
        }
    }

    // Last, check against mods in the lock file
    for m in locked {
        let pid = m.project();
        let vid = &m.id.version;

        // If locked mod isn't in profile or it has a different version, delete it
        {
            let ver = versioned.get(pid);
            let pen = pending.get(pid);
            if reset
                || (!unversioned.remove(pid) && ver.is_none() && pen.is_none())
                || ver.into_iter().any(|pv| pv.as_ref() != vid)
                || pen.into_iter().any(|pv| pv.id != vid)
            {
                delete.insert(m.file.clone());
                continue;
            }
        }
        let path = profile_path.join(&m.file);
        if !force && path.exists() && verify_sha1(&m.sha1, &path).await.is_ok_and(identity) {
            versioned.remove(pid);
            pending.remove(pid);
            installed.push(m.into());
        } else if !pending.contains(pid) {
            versioned.insert(pid.into(), vid.into());
        }
    }
    resolved
}

async fn install_version(profile_path: &PathAbsolute, v: Version, cached: &Path) -> Result<LockedMod> {
    let lm = {
        let mut lm: LockedMod = v.into();
        // put in mods subdir if not specified
        lm.file = match lm.file.parent() {
            Some(p) if !p.as_os_str().is_empty() => lm.file,
            _ => MODS_PATH.join(lm.file),
        };

        lm
    };

    let dest = profile_path.join(&lm.file);
    if cached != &*dest {
        let _ = tokio::fs::create_dir_all(dest.parent().expect("dest directory should always be valid")).await;
        tokio::fs::copy(cached, dest)
            .await
            .with_context(|| format!("Failed to copy downloaded mod into profile: {}", lm.file.display()))?;
    }
    Ok(lm)
}


#[derive(Debug, Default)]
struct ResolvedMods<'a> {
    versioned: HashMap<Cow<'a, ProjectId>, Cow<'a, VersionId>>,
    unversioned: HashSet<Cow<'a, ProjectId>>,
    installed: Vec<Cow<'a, LockedMod>>,
    pending: VersionSet,
}
