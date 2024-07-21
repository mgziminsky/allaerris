use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    convert::identity,
    fmt::Write,
    ops::Deref,
    path::Path,
};

use anyhow::{anyhow, Context};
use async_scoped::TokioScope;
use once_cell::sync::Lazy;

use crate::{
    checked_types::{PathAbsolute, PathScoped, PathScopedRef},
    client::schema::{ProjectId, Version, VersionId},
    config::{profile::ProfileData, Mod, Profile, VersionedProject},
    mgmt::{
        cache,
        events::{EventSouce, InstallType, ProgressEvent},
        hash::{verify_sha1, verify_sha1_sync, Sha1Writer},
        lockfile::{LockFile, LockedId, LockedMod, LockedPack},
        modpack::ModpackData,
        version::VersionSet,
        ProfileManager,
    },
    Client, ErrorKind, Result, StdResult,
};

type Downloads = Vec<StdResult<Option<(Version, PathAbsolute)>, tokio::task::JoinError>>;


static MODS_PATH: Lazy<&PathScopedRef> = Lazy::new(|| PathScopedRef::new("mods").unwrap());

impl ProfileManager {
    /// Attempt to download and install all mods defined in
    /// [`profile`](Profile).
    ///
    /// Any previously installed mods that were removed from the profile will be
    /// deleted. Updated versions of mods will delete the old version after
    /// installing the update.
    ///
    /// Will return a the list of all successfully installed mods. Progress and
    /// most errors will be sent to [`channel`]
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

        self.send(ProgressEvent::Status("Loading lockfile...".to_string()));
        let mut lockfile = LockFile::load(profile.path()).await?;
        let reset = lockfile.game_version != data.game_version || lockfile.loader != data.loader;
        lockfile.game_version = data.game_version.clone();
        lockfile.loader = data.loader;

        // This will be passed to other local methods which will add to it any files
        // that should be deleted
        let mut delete = vec![];

        let mut pack = self.load_pack(&mut lockfile, data, client, &mut delete).await;

        self.send(ProgressEvent::Status("Resolving mod versions...".to_string()));
        let ResolvedMods {
            versioned,
            unversioned,
            installed,
            mut pending,
        } = merge_sources(&data.mods, &lockfile.mods, pack.as_mut(), profile.path(), &mut delete, reset).await;

        self.send(ProgressEvent::Status("Fetch version details...".to_string()));
        self.fetch_versions(client, data, unversioned, versioned, &mut pending).await;

        self.send(ProgressEvent::Status("Downloading...".to_string()));
        let ((), downloads) = TokioScope::scope_and_block(|scope| {
            let sub = PathScopedRef::new("mods").ok();
            for v in pending {
                let save_path = cache::version_path(&v, v.filename.parent().and_then(|p| p.file_name_path()).or(sub));
                scope.spawn(async { self.dl_version(v.into(), &save_path).await.map(|v| (v, save_path)) });
            }
        });

        // Start deleting old files while new ones install
        let delete_job = self.spawn_delete_files(&delete, profile.path());

        self.send(ProgressEvent::Status("Installing...".to_string()));
        lockfile.mods = installed
            .into_iter()
            .inspect(|m| {
                self.send(ProgressEvent::Installed {
                    file: m.file.clone(),
                    is_new: false,
                    typ: InstallType::Mod,
                })
            })
            .map(Cow::into_owned)
            .collect();
        lockfile.mods.extend(self.install_downloaded(downloads, profile.path()).await);

        if delete_job.await.is_err() {
            let mut files = String::with_capacity(2 * delete.len() + delete.iter().map(|p| p.as_os_str().len()).sum::<usize>());
            for file in delete {
                files.push_str("\n\t");
                let _ = write!(&mut files, "{}", file.display());
            }
            self.send_err(anyhow!("Unexpected error deleting old files. The following may need deleted manually:{files}",).into())
        }

        if let Some(pack) = pack {
            self.extract_overrides(&mut lockfile, pack, profile.path());
        }

        lockfile.mods.sort_unstable_by(|a, b| a.file.cmp(&b.file));
        if let Err(e) = lockfile.save(profile.path()).await {
            self.send_err(e);
        };

        Ok(())
    }

    fn extract_overrides(&self, lockfile: &mut LockFile, mut pack: ModpackData, profile_path: &PathAbsolute) {
        self.send(ProgressEvent::Status("Modpack Overrides...".to_string()));
        let Some(LockedPack { ref mut overrides, .. }) = lockfile.pack.as_mut() else {
            unreachable!("Should have lockfile pack if we have pack data");
        };
        pack.visit_overrides(|path, mut file| {
            macro_rules! event {
                ($new:literal) => {
                    ProgressEvent::Installed {
                        file: path.to_owned(),
                        is_new: $new,
                        typ: InstallType::Override,
                    }
                };
            }

            let target = &profile_path.join(path);
            if let Some(sha1) = overrides.get(path) {
                if let Ok(true) = verify_sha1_sync(sha1, target) {
                    self.send(event!(false));
                    return;
                };
            }

            use std::{fs, io};
            target.parent().map(fs::create_dir_all);
            let sha1 = fs::File::create(target).and_then(|target| {
                let mut target = Sha1Writer::new(target);
                io::copy(&mut file, &mut target)?;
                target.finalize_str()
            });
            match sha1 {
                Ok(sha1) => {
                    overrides.insert(path.to_owned(), sha1);
                    self.send(event!(true))
                },
                Err(e) => self.send_err(e.into()),
            }
        });
    }

    /// Fetches and loads the modpack data from the pack index.
    async fn load_pack(
        &self,
        lockfile: &mut LockFile,
        data: &ProfileData,
        client: &Client,
        delete: &mut Vec<PathScoped>,
    ) -> Option<ModpackData> {
        macro_rules! fetch_replace {
            ($pack:expr) => {
                self.fetch_pack(client, $pack.deref(), data).await.map(|(data, lm)| {
                    lockfile.pack.replace(LockedPack::new(lm));
                    data
                })
            };
        }
        // Marks unchanged overrides for deletion
        macro_rules! delete_overrides {
            ($lp:expr) => {
                for (p, s) in std::mem::take(&mut $lp.overrides) {
                    if let Ok(true) = verify_sha1(&s, &p).await {
                        delete.push(p);
                    } else {
                        self.send(ProgressEvent::Status(format!(
                            "Keeping changed override file: {}",
                            p.display()
                        )));
                    }
                }
            };
        }
        match (lockfile.pack.as_mut(), &data.modpack) {
            (None, None) => None,
            (Some(lp), None) => {
                delete_overrides!(lp);
                None
            },
            (Some(lp), Some(pack)) => {
                if pack.version().is_some_and(|v| v != &lp.id.version) {
                    delete_overrides!(lp);
                    fetch_replace!(pack)
                } else {
                    if !pack.install_overrides {
                        delete_overrides!(lp);
                    }
                    self.fetch_pack(client, lp, data).await.map(|(data, _)| data)
                }
            },
            (None, Some(pack)) => fetch_replace!(pack),
        }
    }

    async fn fetch_pack(&self, client: &Client, pack: &impl VersionedProject, data: &ProfileData) -> Option<(ModpackData, LockedMod)> {
        self.send(ProgressEvent::Status("Fetch and read modpack...".to_string()));
        match self.load_modpack(client, pack, data).await {
            Ok((v, data)) => {
                let lm = LockedMod {
                    id: LockedId {
                        project: v.project_id,
                        version: v.id,
                    },
                    file: v.filename,
                    sha1: v.sha1.expect("Downloaded pack should have a sha1"),
                };
                Some((data, lm))
            },
            Err(e) => {
                self.send_err(e);
                None
            },
        }
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
        // Get the latest version of all unversioned projects
        let ((), pending) = TokioScope::scope_and_block(|scope| {
            for id in unversioned.iter() {
                scope.spawn(client.get_latest(id.as_ref(), Some(&data.game_version), Some(data.loader)));
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

    async fn install_downloaded(&self, downloads: Downloads, profile_path: &Path) -> Vec<LockedMod> {
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

    fn spawn_delete_files(&self, delete: &[PathScoped], profile_path: &Path) -> tokio::task::JoinHandle<()> {
        let delete = delete.iter().map(|p| (p.to_owned(), profile_path.join(p))).collect::<Vec<_>>();
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
}

async fn merge_sources<'a>(
    profile: &'a Vec<Mod>,
    locked: &'a Vec<LockedMod>,
    pack: Option<&'a mut ModpackData>,
    profile_path: &'a Path,
    delete: &mut Vec<PathScoped>,
    reset: bool,
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
    use crate::mgmt::modpack::PackMods::*;
    match pack.map(|p| &mut p.mods) {
        Some(Modrinth { known, .. }) => std::mem::swap(pending, known),
        Some(Forge(mods)) => std::mem::swap(versioned, &mut mods.iter().map(|(k, v)| (k.into(), v.into())).collect()),
        None => {},
    }

    // Then mods from the profile
    for m in profile {
        let pid = m.project();
        match (pending.get(pid), m.version()) {
            (None, None) => {
                unversioned.insert(pid.into());
            },
            (None, Some(v)) => {
                versioned.insert(pid.into(), v.into());
            },
            (Some(mpv), Some(v)) if &mpv.id != v => {
                versioned.insert(pid.into(), v.into());
                pending.remove(pid);
            },
            (Some(_), Some(_)) => { /* Versions match - Keep modpack version */ },
            (Some(_), None) => { /* Keep modpack version */ },
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
                || pen.into_iter().any(|pv| &pv.id != vid)
            {
                delete.push(m.file.clone());
                continue;
            }
        }
        let path = profile_path.join(&m.file);
        if path.exists() && verify_sha1(&m.sha1, &path).await.is_ok_and(identity) {
            versioned.remove(pid);
            pending.remove(pid);
            installed.push(m.into());
        } else if !pending.contains(pid) {
            versioned.insert(pid.into(), vid.into());
        }
    }
    resolved
}

async fn install_version(profile_path: &Path, v: Version, cached: &Path) -> Result<LockedMod> {
    // put in mods subdir if not specified
    let file = match v.filename.parent() {
        Some(p) if !p.as_os_str().is_empty() => v.filename,
        _ => MODS_PATH.join(v.filename),
    };
    let lm = LockedMod {
        id: LockedId::new(v.project_id, v.id)?,
        sha1: v.sha1.expect("downloaded mod should always have a sha1"),
        file,
    };
    let dest = profile_path.join(&lm.file);
    let _ = tokio::fs::create_dir_all(dest.parent().expect("dest directory should always be valid")).await;
    tokio::fs::copy(cached, dest)
        .await
        .with_context(|| format!("Failed to copy downloaded mod into profile: {}", lm.file.display()))?;
    Ok(lm)
}

#[derive(Debug, Default)]
struct ResolvedMods<'a> {
    versioned: HashMap<Cow<'a, ProjectId>, Cow<'a, VersionId>>,
    unversioned: HashSet<Cow<'a, ProjectId>>,
    installed: Vec<Cow<'a, LockedMod>>,
    pending: VersionSet,
}
