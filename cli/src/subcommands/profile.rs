mod configure;
mod create;
mod delete;
mod helpers;

use anyhow::{bail, Context, Result};
use relibium::{
    checked_types::PathAbsolute,
    config::{profile::ProfileData, Profile},
    Config,
};
use yansi::Paint;

use self::{configure::configure, create::create, delete::delete, helpers::switch_profile};
use crate::{
    cli::ProfileSubCommand,
    helpers::{consts, get_active_profile},
    tui::{self, fmt_profile_simple},
};


pub async fn process(subcommand: ProfileSubCommand, config: &mut Config) -> Result<()> {
    match subcommand {
        ProfileSubCommand::Info => {
            tui::print_profile(get_active_profile(config)?, true).await;
        },
        ProfileSubCommand::List => {
            if let Some(active) = config.active() {
                let mut profiles = config.get_profiles();
                profiles.sort_by_cached_key(|p| p.name().to_lowercase());
                for p in profiles {
                    tui::print_profile(p, p.path() == active).await;
                }
            }
        },
        ProfileSubCommand::New {
            game_version,
            loader,
            name,
            path,
        } => {
            create(config, game_version, loader, name, path).await?;
            println!(
                "{}",
                format!(
                    "After adding your mods, remember to run `{}` to download them!",
                    concat!(consts!(APP_NAME), " upgrade").bold()
                )
                .yellow()
                .wrap()
            );
        },
        ProfileSubCommand::Add { name, path } => {
            let path = PathAbsolute::new(path)?;
            if !ProfileData::file_path(&path).exists() {
                bail!(
                    "No existing profile found at `{}`\nUse `{}` to create one",
                    path.display().bold().italic(),
                    concat!(consts!(APP_NAME), " new").bold(),
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
        ProfileSubCommand::Remove { profile_name, switch_to } => {
            let removed = delete(config, profile_name, switch_to)?;
            println!("Profile Removed: {}", fmt_profile_simple(&removed, 100));
            if let Ok(active) = config.active_profile() {
                println!("Active Profile:  {}", fmt_profile_simple(active, 100));
            }
        },
        ProfileSubCommand::Configure {
            game_version,
            loader,
            name,
        } => {
            configure(get_active_profile(config)?, game_version, loader, name).await?;
        },
        ProfileSubCommand::Switch { profile_name } => {
            let profiles = config.get_profiles();
            switch_profile!(config, profiles, profile_name);
        },
    };
    Ok(())
}
