use std::{collections::HashSet, fmt::Display};

use anyhow::Result;
use ferrallay::{
    Client,
    client::schema::{ProjectIdSvcType, VersionIdSvcType},
    config::{Mod, Profile, VersionedProject},
};
use yansi::{Paint, Style};

use crate::tui::{self, const_style};

const ID_STYLE: Style = Style::new().bright().bold();
const PREV_STYLE: Style = Style::new().bright_red().italic().strike();

macro_rules! mods {
    ($profile:expr) => {
        $profile.data_mut().await?.mods.iter_mut()
    };
}
macro_rules! take_else_cont {
    ($id:expr, $mods:expr $(, $else:expr)?$(,)?) => {{
        let Some(res) = $mods.iter().position(|__| __.project() == $id) else {
            $(let _ = { $else };)?
            continue;
        };
        $mods.swap_remove(res)
    }};
}
macro_rules! not_found_msg {
    ($id:expr) => {
        format_args!("No mod or pack with id `{}` found in profile", ($id.paint(ID_STYLE)))
            .yellow()
            .wrap()
    };
}

/// Lock all mods/pack with `ids` to their installed version unless already
/// locked. If `force` is true, replace any already locked mods with their
/// installed version.
///
/// # Errors
/// - Reading the [profile data](ferrallay::config::profile::ProfileData) fails
/// - Reading the lockfile with installed mod versions fails
pub async fn lock_mods(profile: &mut Profile, ids: &[impl ProjectIdSvcType + Display], force: bool) -> Result<()> {
    let mut installed = profile.installed().await?;
    let (mods, missing) = {
        let mut missing = vec![];
        let mut mods = HashSet::with_capacity(ids.len());
        let mut all = mods!(profile).collect::<Vec<_>>();
        for id in ids {
            let m = take_else_cont!(id, all, missing.push(id));
            mods.insert(m);
        }
        (mods, missing)
    };
    lock(mods, &mut installed, force);
    if !missing.is_empty() {
        eprintln!("The following IDs were not found in the active profile:");
        for id in missing {
            eprintln!("    {}", id.paint(ID_STYLE).red());
        }
    }
    Ok(())
}

pub async fn lock_versions(profile: &mut Profile, client: &Client, ids: &[impl VersionIdSvcType + Display], force: bool) -> Result<()> {
    let mut versions = client.get_versions(&ids.iter().map(|id| id as _).collect::<Vec<_>>()).await?;
    let mods = {
        let pids = versions.iter().map(VersionedProject::project).collect::<HashSet<_>>();
        mods!(profile).filter(|m| pids.contains(m.project())).collect::<Vec<_>>()
    };
    lock(mods, &mut versions, force);
    // Anything left in versions isn't in the profile
    if !versions.is_empty() {
        eprintln!("No mods or pack for the following versions were found in the active profile:");
        for v in versions {
            eprintln!("    {}", format_args!("{} ― {}", tui::vid_tag(&v.id), v.title.bold()));
        }
    }

    Ok(())
}

/// Locks modpack and all mods to their installed version. Already locked mods
/// will only be updated if `force` is true
pub async fn lock_all(profile: &mut Profile, force: bool) -> Result<()> {
    let mut installed = profile.installed().await?;
    lock(mods!(profile), &mut installed, force);

    Ok(())
}

fn lock<'m>(mods: impl IntoIterator<Item = &'m mut Mod>, versions: &mut Vec<impl VersionedProject>, force: bool) {
    let (update, keep): (Vec<_>, Vec<_>) = mods.into_iter().partition(|m| force || m.version().is_none());
    let mut missing = vec![];
    for m in update {
        let new = take_else_cont!(m.project(), versions, missing.push(&*m));
        let vid = new.version().expect("All values from `versions` should have a version id");
        let old = m.id.set_version(vid.clone()).unwrap();

        print!("{}: {}", tui::mod_single_line(m), vid.bright_blue().italic());
        match old {
            Some(v) if v != vid => print!(" (Previous: {})", v.paint(PREV_STYLE)),
            _ => (),
        }
        println!();
    }
    if !keep.is_empty() {
        eprintln!(
            "The following mods are already locked and were not updated. Use the {} option to override",
            const_style!("--force"; yellow().bold())
        );
        for m in keep {
            eprintln!("    {}", tui::mod_single_line(m));
        }
    }
    if !missing.is_empty() {
        eprintln!(
            "The following mods are not yet installed and were not locked. Run {} to install them",
            const_style!("apply"; yellow().bold())
        );
        for m in missing {
            eprintln!("    {}", tui::mod_single_line(m));
        }
    }
}

/// Remove the version from the projects with `ids`
pub async fn unlock(profile: &mut Profile, ids: &[impl ProjectIdSvcType + Display]) -> Result<()> {
    let mut mods = mods!(profile).collect::<Vec<_>>();
    for id in ids {
        let m = take_else_cont!(id, mods, eprintln!("{}", not_found_msg!(id)));
        print!("{}: ", tui::mod_single_line(m));
        if let Some(v) = m.id.unset_version() {
            println!("{}", v.paint(PREV_STYLE));
        } else {
            println!("No Change");
        }
    }

    Ok(())
}

/// Remove the version from the mod/pack with `id` returning it's previous value
pub async fn unlock_all(profile: &mut Profile) -> Result<()> {
    let mut all = mods!(profile).filter(|m| m.version().is_some()).peekable();
    if all.peek().is_none() {
        println!("All mods already unlocked");
    } else {
        for m in all {
            let v = m.id.unset_version().unwrap();
            println!("{}: {}", tui::mod_single_line(m), v.paint(PREV_STYLE));
        }
    }

    Ok(())
}
