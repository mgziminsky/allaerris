mod cli;
// mod download;
mod file_picker;
mod subcommands;
mod tui;

use anyhow::{anyhow, bail, Context, Result};
use clap::{CommandFactory, Parser};
use colored::Colorize;
use relibium::{
    client::{Client, ForgeClient, GithubClient, ModrinthClient},
    config::{Config, Profile, DEFAULT_CONFIG_PATH},
    curseforge::client::AuthData,
};
use std::{
    env::{var, var_os},
    ffi::OsStr,
    path::Path,
    process::ExitCode,
};
use tokio::runtime;

use self::{
    cli::{Ferium, ModpackSubCommands, ProfileSubCommands, SubCommands},
    subcommands::profile::switch_profile,
    tui::{fmt_profile_simple, print_mods},
};

macro_rules! consts {
    (APP_NAME) => {
        env!("CARGO_PKG_NAME")
    };
}
const APP_NAME: &str = consts!(APP_NAME);

const USER_AGENT: &str = concat!(
    consts!(APP_NAME),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (Github: mgziminsky)"
);

const MSG_PROFILE_EMPTY: &str = concat!(
    "The currently selected profile is empty! Run `",
    consts!(APP_NAME),
    " help` to see how to add mods"
);

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
            eprintln!(
                "{}",
                "Verify that you are connnected to the internet"
                    .yellow()
                    .bold()
            );
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
    // Alias `ferium modpacks` to `ferium modpack list`
    if let SubCommands::Modpacks = cli_app.subcommand {
        cli_app.subcommand = SubCommands::Modpack {
            subcommand: Some(ModpackSubCommands::List),
        };
    }

    let client: Client = vec![
        ModrinthClient::builder()
            .user_agent(USER_AGENT)
            .build()?
            .into(),
        ForgeClient::builder()
            .user_agent(USER_AGENT)
            .auth(AuthData {
                api_key_auth: Some(
                    cli_app
                        .curseforge_api_key
                        .or_else(|| var("CURSEFORGE_API_KEY").ok())
                        .unwrap_or(
                            "FIXME-GET-FORGE-KEY"
                                .to_owned(),
                        ),
                ),
            })
            .build()?
            .into(),
        GithubClient::builder()
            .personal_token(
                cli_app
                    .github_token
                    .or_else(|| var("GITHUB_TOKEN").ok())
                    .unwrap_or_default(),
            )
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
        SubCommands::Complete { .. } | SubCommands::Profiles | SubCommands::Modpacks => {
            unreachable!();
        }
        SubCommands::List { verbose, markdown } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile).await?;
            if verbose {
                subcommands::list::verbose(&client, profile, markdown).await?;
            } else {
                subcommands::list::simple(profile).await?;
            }
        }
        SubCommands::Add { identifiers: ids } => {
            if ids.is_empty() {
                bail!("Must provide at least one identifier")
            }
            subcommands::add(
                client,
                get_active_profile(&mut config)?.data_mut().await?,
                ids,
            )
            .await?;
        }
        SubCommands::Remove { mod_names } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile).await?;
            let removed = subcommands::remove(profile.data_mut().await?, mod_names)?;
            if !removed.is_empty() {
                print_mods(
                    format_args!(
                        "Removed {} Mods",
                        format!("{}", removed.len()).yellow().bold()
                    ),
                    &removed,
                );
            }
        }
        SubCommands::Profile { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ProfileSubCommands::Info
            });
            match subcommand {
                ProfileSubCommands::Info => {
                    tui::print_profile(get_active_profile(&mut config)?, true).await;
                }
                ProfileSubCommands::Configure {
                    game_version,
                    loader,
                    name,
                } => {
                    subcommands::profile::configure(
                        get_active_profile(&mut config)?,
                        game_version,
                        loader,
                        name,
                    )
                    .await?;
                }
                ProfileSubCommands::Add {
                    game_version,
                    loader,
                    name,
                    path,
                } => {
                    subcommands::profile::create(
                        &client,
                        &mut config,
                        game_version,
                        loader,
                        name,
                        path,
                    )
                    .await?;
                    println!(
                        "{}",
                        format!(
                            "After adding your mods, remember to run `{}` to download them!",
                            concat!(consts!(APP_NAME), " upgrade").bold()
                        )
                        .yellow()
                    );
                }
                ProfileSubCommands::Remove {
                    profile_name,
                    switch_to,
                } => {
                    let removed =
                        subcommands::profile::delete(&mut config, profile_name, switch_to)?;
                    println!("Profile Removed: {}", fmt_profile_simple(&removed, 100));
                    if let Ok(active) = config.active_profile() {
                        println!("Active Profile:  {}", fmt_profile_simple(active, 100));
                    }
                }
                ProfileSubCommands::List => {
                    if let Some(active) = config.active() {
                        let mut profiles = config.get_profiles();
                        profiles.sort_by_cached_key(|p| p.name().to_lowercase());
                        for p in profiles {
                            tui::print_profile(p, p.path() == active).await;
                        }
                    }
                }
                ProfileSubCommands::Switch { profile_name } => {
                    let profiles = config.get_profiles();
                    switch_profile!(config, profiles, profile_name);
                }
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
        }
        _ => todo!(),
        // SubCommands::Upgrade => {
        //     let profile = get_active_profile(&mut config).await?;
        //     check_empty_profile(profile)?;
        //     subcommands::upgrade(modrinth, curseforge, github, profile).await?;
        // }
        // SubCommands::Modpack { subcommand } => {
        //     let mut default_flag = false;
        //     let subcommand = subcommand.unwrap_or_else(|| {
        //         default_flag = true;
        //         ModpackSubCommands::Info
        //     });
        //     match subcommand {
        //         ModpackSubCommands::Add {
        //             identifier,
        //             output_dir,
        //             install_overrides,
        //         } => {
        //             subcommands::modpack::add(
        //                 identifier,
        //                 &mut config,
        //                 output_dir,
        //                 install_overrides,
        //                 &curseforge,
        //                 &modrinth,
        //             )
        //             .await?;
        //         }
        //         ModpackSubCommands::Configure {
        //             output_dir,
        //             install_overrides,
        //         } => {
        //             subcommands::modpack::configure(
        //                 get_active_modpack(&mut config)?,
        //                 output_dir,
        //                 install_overrides,
        //             )?;
        //         }
        //         ModpackSubCommands::Delete { force } => {
        //             subcommands::modpack::delete(&mut config, force)?;
        //         }
        //         ModpackSubCommands::Info => {
        //             subcommands::modpack::info(get_active_modpack(&mut config)?, true);
        //         }
        //         ModpackSubCommands::List => {
        //             config.modpacks().for_each(|(i, p)| {
        //                 subcommands::profile::info(p, i == config.active_profile)
        //             });
        //         }
        //         ModpackSubCommands::Upgrade => {
        //             subcommands::modpack::upgrade(
        //                 &modrinth,
        //                 &curseforge,
        //                 get_active_modpack(&mut config)?,
        //             )
        //             .await?;
        //         }
        //     };
        //     if default_flag {
        //         println!(
        //             "{} ferium modpack help {}",
        //             "Use".yellow(),
        //             "for more information about this subcommand".yellow()
        //         );
        //     }
        // }
    };

    config.save_to(config_path).await?;

    Ok(())
}

/// Get the active profile with error handling
fn get_active_profile(config: &mut Config) -> Result<&mut Profile> {
    config
        .active_profile_mut()
        .map_err(|err| match err.kind() {
            relibium::ErrorKind::NoProfiles => anyhow!(
                "There are no profiles configured, add a profile using `{}`",
                format!("{APP_NAME} profile create").bold().italic()
            ),
            _ => err.into(),
        })
        .with_context(|| "Failed to load active profile".bold())
}

/// Check if `profile` is empty, and if so return an error
async fn check_empty_profile(profile: &Profile) -> Result<()> {
    if profile.data().await?.is_empty() {
        bail!(MSG_PROFILE_EMPTY);
    }
    Ok(())
}
