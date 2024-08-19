use std::collections::{HashMap, HashSet};

use crate::{
    checked_types::PathScoped,
    client::schema::Version,
    config::Profile,
    mgmt::{events::EventSouce, lockfile::LockFile},
    Client, ProfileManager, Result,
};

impl ProfileManager {
    /// Lookup all unknown files non-recursively in the profile mods folder
    #[allow(clippy::missing_panics_doc)]
    pub async fn scan(&self, client: &Client, profile: &Profile, all: bool) -> Result<HashMap<PathScoped, Version>> {
        let profile_path = profile.path();
        let locked = if all {
            HashSet::new()
        } else {
            LockFile::load(profile_path)
                .await?
                .mods
                .into_iter()
                .map(|lm| profile_path.join(lm.file))
                .collect()
        };

        let mut files = tokio::fs::read_dir(profile_path.join("mods")).await?;
        let mut paths = vec![];
        while let Some(entry) = files.next_entry().await.transpose() {
            match entry {
                Ok(e) if e.file_type().await.is_ok_and(|ft| ft.is_file()) => {
                    let path = e.path();
                    if !locked.contains(path.as_path()) {
                        paths.push(path);
                    }
                },
                Err(err) => self.send_err(err.into()),
                Ok(_) => { /* Non file */ },
            }
        }

        let mut results = HashMap::new();
        client.lookup(&paths, &mut results).await?;
        let results = results
            .into_iter()
            .map(|(k, v)| {
                (
                    k.strip_prefix(profile_path)
                        // SAFETY: read_dir is called on a PathAbsolute and strip_prefix produces a relative path
                        .map(|p| unsafe { PathScoped::new_unchecked(p) })
                        .expect("scanned path should be prefixed by profile path"),
                    v,
                )
            })
            .collect();

        Ok(results)
    }
}
