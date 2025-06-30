mod configure;
mod create;
mod delete;
mod helpers;

use anyhow::{Context, Result, bail};
use ferrallay::{
    Config,
    checked_types::PathAbsolute,
    config::{Profile, profile::ProfileData},
};
use yansi::Paint;

pub use self::configure::configure;
use self::{create::create, delete::delete, helpers::switch_profile};
use crate::{
    cli::ProfileSubcommand,
    helpers::{consts, get_active_profile},
    tui::{self, fmt_profile_simple},
};


pub async fn process(subcommand: ProfileSubcommand, config: &mut Config) -> Result<()> {
    match subcommand {
        ProfileSubcommand::Info => {
            tui::print_profile(get_active_profile(config)?, true).await;
        },
        ProfileSubcommand::List => {
            if let Some(active) = config.active() {
                let mut profiles = config.get_profiles();
                profiles.sort_by_cached_key(|p| p.name().to_lowercase());
                for p in profiles {
                    tui::print_profile(p, p.path() == active).await;
                }
            }
        },
        ProfileSubcommand::New {
            game_version,
            loader,
            name,
            path,
            server,
        } => {
            create(config, game_version, loader, name, path, server).await?;
            println!(
                "{}",
                format!(
                    "After adding your mods, remember to run `{}` to download them!",
                    concat!(consts!(APP_NAME), " apply").bold()
                )
                .yellow()
                .wrap()
            );
        },
        ProfileSubcommand::Import { name, path } => {
            let path = PathAbsolute::new(path)?;
            if !ProfileData::file_path(&path).exists() {
                bail!(
                    "No existing profile found at `{}`\nUse `{}` to create one",
                    path.display().bold().italic(),
                    concat!(consts!(APP_NAME), " profile new").bold(),
                );
            }
            if let Err(prof) = config.add_profile(Profile::new(name, path.clone())) {
                let existing = config.profile(prof.path()).expect("Profile should already exist");
                bail!("Profile already present in config: {}", fmt_profile_simple(existing, 80).bold())
            }
            let _ = config
                .set_active(path)
                .context("Failed to switch to imported profile")
                .inspect_err(|e| eprintln!("{:?}", e.yellow()))
                .inspect(|()| println!("The imported profile is now active"));
        },
        ProfileSubcommand::Remove { profile_name, switch_to } => {
            let removed = delete(config, profile_name, switch_to)?;
            println!("Profile Removed: {}", fmt_profile_simple(&removed, 100));
            if let Ok(active) = config.active_profile() {
                println!("Active Profile:  {}", fmt_profile_simple(active, 100));
            }
        },
        ProfileSubcommand::Edit {
            game_version,
            loader,
            name,
        } => {
            configure(get_active_profile(config)?, game_version, loader, name).await?;
        },
        ProfileSubcommand::Switch { profile_name } => {
            let profiles = config.get_profiles();
            switch_profile!(config, profiles, profile_name);
        },
    }
    Ok(())
}
