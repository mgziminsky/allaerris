use anyhow::{Result, bail};
use ferrallay::{Client, config::Profile};
use yansi::Paint;

use crate::{cli::ModsSubcommand, helpers, tui::print_mods};

mod add;
mod list;
mod locking;
mod mgmt;
mod remove;

pub async fn process(subcommand: ModsSubcommand, profile: &mut Profile, client: &Client) -> Result<()> {
    use ModsSubcommand::*;
    match subcommand {
        Add { ids, exclude } => {
            if ids.is_empty() {
                bail!("Must provide at least one project ID");
            }
            let new = add::add(client, profile.data_mut().await?, ids, exclude).await?;
            if new > 0 {
                profile.save().await?;
            }
        },
        Remove { mod_names } => {
            helpers::check_empty_profile(profile).await?;
            let removed = remove::remove(profile.data_mut().await?, &mod_names)?;
            if !removed.is_empty() {
                print_mods(format_args!("Removed {} Mods", removed.len().yellow().bold()), &removed);
                profile.save().await?;
            }
        },
        List { verbose, markdown } => {
            helpers::check_empty_profile(profile).await?;
            if verbose || markdown {
                list::verbose(client, profile, markdown).await?;
            } else {
                list::simple(profile).await?;
            }
        },
        Lock { ids, versions, force, all } => {
            if all {
                locking::lock_all(profile, force).await?;
            } else if versions {
                locking::lock_versions(profile, client, &ids, force).await?;
            } else {
                locking::lock_mods(profile, &ids, force).await?;
            }
            profile.save().await?;
        },
        Unlock { ids, all } => {
            if all {
                locking::unlock_all(profile).await?;
            } else {
                locking::unlock(profile, &ids).await?;
            }
            profile.save().await?;
        },
        Mgmt(command) => mgmt::process(command, client, profile).await?,
    }
    Ok(())
}
