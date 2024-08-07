mod cli;
mod file_picker;
mod helpers;
mod subcommands;
mod tui;

use std::{
    env::{var, var_os},
    ffi::OsStr,
    path::Path,
    process::ExitCode,
};

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use relibium::{
    client::{Client, ForgeClient, GithubClient, ModrinthClient},
    config::{profile::ProfileData, Config, Profile, DEFAULT_CONFIG_PATH},
    curseforge::client::AuthData,
};
use tokio::{runtime, sync::OnceCell};
use yansi::Paint;

use self::{
    cli::{Ferium, ModpackSubcommand, ProfileSubcommand, SubCommand},
    helpers::{consts, get_active_profile, APP_NAME},
    subcommands::{modpack, mods, profile},
    tui::const_style,
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
    if let Err(err) = runtime.block_on(actual_main(cli)) {
        eprintln!("{:?}", err.red().wrap());
        if err.to_string().contains("error trying to connect") {
            eprintln!("{}", "Verify that you are connnected to the internet".yellow().bold());
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[allow(clippy::too_many_lines)]
async fn actual_main(mut cli_app: Ferium) -> Result<()> {
    // The complete command should not require a config.
    // See [#139](https://github.com/gorilla-devs/ferium/issues/139) for why this might be a problem.
    if let SubCommand::Complete { shell } = cli_app.subcommand {
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
    // Alias `profiles` to `profile list`
    if let SubCommand::Profiles = cli_app.subcommand {
        cli_app.subcommand = SubCommand::Profile {
            subcommand: Some(ProfileSubcommand::List),
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
        .unwrap_or(DEFAULT_CONFIG_PATH.to_owned().into());

    // This craziness is because I want a lazy, async, and mut config that can be
    // tested for initialization later. Only works as macros since LazyCell
    // isn't mut or testable, mut closures can't return non-local references,
    // and async blocks have same issue as closures and can't be awaited more than
    // once
    let config_ = &mut OnceCell::new();
    /// Lazy load the config from `config_path`
    macro_rules! config {
        () => {{
            config_
                .get_or_init(|| async {
                    Config::load_from(config_path)
                        .await
                        .with_context(|| format!("Failed to read config file at `{}`, using defaults", config_path.display().bold()))
                        .inspect_err(|err| eprintln!("{:?}", err.yellow().wrap()))
                        .unwrap_or_default()
                })
                .await;
            config_.get_mut().unwrap()
        }};
    }
    /// Get the profile of the current working directory, otherwise the active
    /// profile from the config
    macro_rules! profile {
        () => {
            match path_profile().as_mut() {
                Some(prof) => prof,
                None => get_active_profile(config!())?,
            }
        };
    }

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommand::Complete { .. } | SubCommand::Profiles => {
            unreachable!();
        },
        SubCommand::Mods(subcommand) => mods::process(subcommand, profile!(), client).await?,
        SubCommand::Modpack { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ModpackSubcommand::Info
            });
            modpack::process(subcommand, profile!(), client).await?;
            if default_flag {
                println!(
                    "{}",
                    format_args!(
                        "Use `{}` for more information about this subcommand",
                        const_style!(concat!(consts!(APP_NAME), " modpack help"); bold())
                    )
                    .yellow()
                    .wrap()
                );
            }
        },
        SubCommand::Profile { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ProfileSubcommand::Info
            });
            match subcommand {
                ProfileSubcommand::Info => tui::print_profile(profile!(), true).await,
                _ => profile::process(subcommand, config!()).await?,
            }
            if default_flag {
                println!(
                    "{}",
                    format_args!(
                        "Use `{}` for more information about this subcommand",
                        const_style!(concat!(consts!(APP_NAME), " profile help"); bold())
                    )
                    .yellow()
                    .wrap()
                );
            }
        },
    };

    if let Some(config) = config_.get_mut() {
        config.save_to(config_path).await?;
    }

    Ok(())
}

/// Look for a profile file in the current directory and its ancestors
fn path_profile() -> Option<Profile> {
    std::path::absolute(".")
        .ok()
        .as_deref()
        .map(Path::ancestors)
        .and_then(|mut anc| anc.find(|p| ProfileData::file_path(p).exists()))
        .map(|p| Profile::new("Local Directory".to_owned(), p.try_into().expect("Should be an absolute path")))
}
