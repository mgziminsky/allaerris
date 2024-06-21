mod cli;
mod helpers;
// mod download;
mod file_picker;
mod subcommands;
mod tui;

use std::{
    env::{var, var_os},
    ffi::OsStr,
    path::Path,
    process::ExitCode,
};

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser};
use colored::Colorize;
use relibium::{
    client::{Client, ForgeClient, GithubClient, ModrinthClient},
    config::{profile::ProfileData, Config, Profile, DEFAULT_CONFIG_PATH},
    curseforge::client::AuthData,
};
use tokio::runtime;

use self::{
    cli::{Ferium, ModpackSubCommands, ProfileSubCommands, SubCommands},
    helpers::{consts, APP_NAME},
    subcommands::{
        modpack,
        profile::{normalize_profile_path, switch_profile},
    },
    tui::{fmt_profile_simple, print_mods},
};

const USER_AGENT: &str = concat!(consts!(APP_NAME), "/", env!("CARGO_PKG_VERSION"), " (Github: mgziminsky)");

fn main() -> ExitCode {
    let cli = Ferium::parse();
    let runtime = {
        let mut builder = runtime::Builder::new_multi_thread();
        builder.enable_all();
        builder.thread_name("ferium-worker");
        if let Some(threads) = cli.threads {
            builder.worker_threads(threads);
        }
        builder.build().expect("Could not initialise Tokio runtime")
    };
    #[cfg(windows)]
    {
        // Enable colours on conhost
        let _ = colored::control::set_virtual_terminal(true);
    }
    if let Err(err) = runtime.block_on(actual_main(cli)) {
        eprintln!("{}", format!("{err:?}").red());
        if err.to_string().contains("error trying to connect") {
            eprintln!("{}", "Verify that you are connnected to the internet".yellow().bold());
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn actual_main(mut cli_app: Ferium) -> Result<()> {
    // The complete command should not require a config.
    // See [#139](https://github.com/gorilla-devs/ferium/issues/139) for why this might be a problem.
    if let SubCommands::Complete { shell } = cli_app.subcommand {
        clap_complete::generate(
            shell,
            &mut Ferium::command(),
            std::env::current_exe()
                .ok()
                .as_deref()
                .and_then(Path::file_name)
                .and_then(OsStr::to_str)
                .unwrap_or(APP_NAME),
            &mut std::io::stdout(),
        );
        return Ok(());
    }
    // Alias `ferium profiles` to `ferium profile list`
    if let SubCommands::Profiles = cli_app.subcommand {
        cli_app.subcommand = SubCommands::Profile {
            subcommand: Some(ProfileSubCommands::List),
        };
    }

    let client: Client = vec![
        ModrinthClient::builder().user_agent(USER_AGENT).build()?.into(),
        ForgeClient::builder()
            .user_agent(USER_AGENT)
            .auth(AuthData {
                api_key_auth: Some(
                    cli_app
                        .curseforge_api_key
                        .or_else(|| var("CURSEFORGE_API_KEY").ok())
                        .unwrap_or("FIXME-GET-FORGE-KEY".to_owned()),
                ),
            })
            .build()?
            .into(),
        GithubClient::builder()
            .personal_token(cli_app.github_token.or_else(|| var("GITHUB_TOKEN").ok()).unwrap_or_default())
            .build()?
            .into(),
    ]
    .try_into()?;

    let config_path = &cli_app
        .config_file
        .or_else(|| var_os("FERIUM_CONFIG_FILE").map(Into::into))
        .unwrap_or(DEFAULT_CONFIG_PATH.to_owned());
    let mut config = Config::load_from(config_path)
        .await
        .with_context(|| {
            format!(
                "Failed to read config file at `{}`, using defaults",
                config_path.display().to_string().bold()
            )
            .yellow()
        })
        .inspect_err(|err| eprintln!("{err}"))
        .unwrap_or_default();

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommands::Complete { .. } | SubCommands::Profiles => {
            unreachable!();
        },
        SubCommands::List { verbose, markdown } => {
            let profile = helpers::get_active_profile(&mut config)?;
            helpers::check_empty_profile(profile).await?;
            if verbose || markdown {
                subcommands::list::verbose(&client, profile, markdown).await?;
            } else {
                subcommands::list::simple(profile).await?;
            }
        },
        SubCommands::Add { identifiers: ids } => {
            if ids.is_empty() {
                bail!("Must provide at least one identifier")
            }
            subcommands::add(client, helpers::get_active_profile(&mut config)?.data_mut().await?, ids).await?;
        },
        SubCommands::Remove { mod_names } => {
            let profile = helpers::get_active_profile(&mut config)?;
            helpers::check_empty_profile(profile).await?;
            let removed = subcommands::remove(profile.data_mut().await?, mod_names)?;
            if !removed.is_empty() {
                print_mods(
                    format_args!("Removed {} Mods", format!("{}", removed.len()).yellow().bold()),
                    &removed,
                );
            }
        },
        SubCommands::Profile { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ProfileSubCommands::Info
            });
            match subcommand {
                ProfileSubCommands::Info => {
                    tui::print_profile(helpers::get_active_profile(&mut config)?, true).await;
                },
                ProfileSubCommands::List => {
                    if let Some(active) = config.active() {
                        let mut profiles = config.get_profiles();
                        profiles.sort_by_cached_key(|p| p.name().to_lowercase());
                        for p in profiles {
                            tui::print_profile(p, p.path() == active).await;
                        }
                    }
                },
                ProfileSubCommands::New {
                    game_version,
                    loader,
                    name,
                    path,
                } => {
                    subcommands::profile::create(&client, &mut config, game_version, loader, name, path).await?;
                    println!(
                        "{}",
                        format!(
                            "After adding your mods, remember to run `{}` to download them!",
                            concat!(consts!(APP_NAME), " upgrade").bold()
                        )
                        .yellow()
                    );
                },
                ProfileSubCommands::Add { name, path } => {
                    let path = normalize_profile_path(path)?;
                    if !ProfileData::file_path(&path).exists() {
                        bail!(
                            "No existing profile found at `{}`\nUse `{}` to create one",
                            path.display().to_string().bold().italic(),
                            concat!(consts!(APP_NAME), " new").bold(),
                        );
                    }
                    if let Err(prof) = config.add_profile(Profile::new(name, path)?) {
                        let existing = config.profile(prof.path()).expect("Profile should already exist");
                        bail!("Profile already present in config: {}", fmt_profile_simple(existing, 80).bold())
                    }
                },
                ProfileSubCommands::Remove { profile_name, switch_to } => {
                    let removed = subcommands::profile::delete(&mut config, profile_name, switch_to)?;
                    println!("Profile Removed: {}", fmt_profile_simple(&removed, 100));
                    if let Ok(active) = config.active_profile() {
                        println!("Active Profile:  {}", fmt_profile_simple(active, 100));
                    }
                },
                ProfileSubCommands::Configure {
                    game_version,
                    loader,
                    name,
                } => {
                    subcommands::profile::configure(&client, helpers::get_active_profile(&mut config)?, game_version, loader, name).await?;
                },
                ProfileSubCommands::Switch { profile_name } => {
                    let profiles = config.get_profiles();
                    switch_profile!(config, profiles, profile_name);
                },
            };
            if default_flag {
                println!(
                    "{}",
                    format!(
                        "Use `{}` for more information about this subcommand",
                        concat!(consts!(APP_NAME), " profile help").bold()
                    )
                    .yellow()
                );
            }
        },
        SubCommands::Modpack { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ModpackSubCommands::Info
            });
            modpack::process(subcommand, &mut config, client).await?;
            if default_flag {
                println!(
                    "{}",
                    format!(
                        "Use `{}` for more information about this subcommand",
                        concat!(consts!(APP_NAME), " modpack help").bold()
                    )
                    .yellow()
                );
            }
        },
        SubCommands::Upgrade => {
            todo!();
            // let profile = get_active_profile(&mut config).await?;
            // check_empty_profile(profile)?;
            // subcommands::upgrade(modrinth, curseforge, github,
            // profile).await?;
        },
    };

    config.save_to(config_path).await?;

    Ok(())
}
