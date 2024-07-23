mod cli;
mod helpers;
// mod download;
mod file_picker;
mod subcommands;
mod tui;

use std::{
    collections::HashMap,
    env::{var, var_os},
    ffi::OsStr,
    path::Path,
    process::ExitCode,
    sync::mpsc,
};

use anyhow::{anyhow, bail, Context, Result};
use clap::{CommandFactory, Parser};
use indicatif::{MultiProgress, ProgressBar};
use relibium::{
    client::{Client, ForgeClient, GithubClient, ModrinthClient},
    config::{Config, DEFAULT_CONFIG_PATH},
    curseforge::client::AuthData,
    mgmt::{
        events::{DownloadId, DownloadProgress, ProgressEvent},
        ProfileManager,
    },
};
use tokio::runtime;
use yansi::Paint;

use self::{
    cli::{Ferium, ModpackSubCommand, ProfileSubCommand, SubCommand},
    helpers::{consts, APP_NAME},
    subcommands::{list, modpack, profile},
    tui::{const_style, print_mods, CROSS_RED, PROG_BYTES, PROG_DONE, TICK_GREEN, TICK_YELLOW},
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
            subcommand: Some(ProfileSubCommand::List),
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
    let mut config = Config::load_from(config_path)
        .await
        .with_context(|| format!("Failed to read config file at `{}`, using defaults", config_path.display().bold()))
        .inspect_err(|err| eprintln!("{:?}", err.yellow().wrap()))
        .unwrap_or_default();

    // Run function(s) based on the sub(sub)command to be executed
    match cli_app.subcommand {
        SubCommand::Complete { .. } | SubCommand::Profiles => {
            unreachable!();
        },
        SubCommand::List { verbose, markdown } => {
            let profile = helpers::get_active_profile(&mut config)?;
            helpers::check_empty_profile(profile).await?;
            if verbose || markdown {
                list::verbose(&client, profile, markdown).await?;
            } else {
                list::simple(profile).await?;
            }
        },
        SubCommand::Add { ids, exclude } => {
            if ids.is_empty() {
                bail!("Must provide at least one identifier")
            }
            subcommands::add(client, helpers::get_active_profile(&mut config)?.data_mut().await?, ids, exclude).await?;
        },
        SubCommand::Remove { mod_names } => {
            let profile = helpers::get_active_profile(&mut config)?;
            helpers::check_empty_profile(profile).await?;
            let removed = subcommands::remove(profile.data_mut().await?, &mod_names)?;
            if !removed.is_empty() {
                print_mods(format_args!("Removed {} Mods", removed.len().yellow().bold()), &removed);
            }
        },
        SubCommand::Profile { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ProfileSubCommand::Info
            });
            profile::process(subcommand, &mut config).await?;
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
        SubCommand::Modpack { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ModpackSubCommand::Info
            });
            modpack::process(subcommand, &mut config, client).await?;
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
        SubCommand::Install => {
            let (sender, handle) = progress_hander();
            ProfileManager::with_channel(sender)
                .apply(&client, helpers::get_active_profile(&mut config)?)
                .await?;
            let _ = handle.await;
        },
    };

    config.save_to(config_path).await?;

    Ok(())
}

pub fn progress_hander() -> (mpsc::Sender<ProgressEvent>, tokio::task::JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    (
        sender,
        tokio::task::spawn_blocking(move || {
            let progress = MultiProgress::new();
            let mut bars = HashMap::new();
            while let Ok(evt) = receiver.recv() {
                match evt {
                    ProgressEvent::Status(msg) => {
                        println!("{msg}");
                    },
                    ProgressEvent::Download(evt) => handle_dl(evt, &mut bars, &progress),
                    ProgressEvent::Installed { file, is_new, typ } => {
                        use relibium::mgmt::events::InstallType::*;
                        println!(
                            "{} {:>9}: {}",
                            if is_new { TICK_GREEN } else { TICK_YELLOW },
                            match typ {
                                Mod => "Installed",
                                Override => "Override",
                                Other => "Other",
                            },
                            file.display()
                        );
                    },
                    ProgressEvent::Deleted(file) => {
                        println!("{}   Deleted: {}", TICK_GREEN, file.display());
                    },
                    ProgressEvent::Error(err) => {
                        eprintln!("{:?}", anyhow!(err).red());
                    },
                }
            }
        }),
    )
}

fn handle_dl(evt: DownloadProgress, bars: &mut HashMap<DownloadId, ProgressBar>, progress: &MultiProgress) {
    use DownloadProgress::*;
    match evt {
        Start { project, title, length } => {
            let bar = progress.add(ProgressBar::new(length).with_message(title).with_style(PROG_BYTES.clone()));
            bars.insert(project, bar);
        },
        Progress(id, len) => {
            if let Some(bar) = bars.get(&id) {
                bar.inc(len);
            }
        },
        Success(id) => {
            if let Some(bar) = bars.remove(&id) {
                bar.with_style(PROG_DONE.clone()).with_prefix(TICK_GREEN.to_string()).finish();
            }
        },
        Fail(id, err) => {
            if let Some(bar) = bars.remove(&id) {
                bar.with_style(PROG_DONE.clone())
                    .with_prefix(CROSS_RED.to_string())
                    .abandon_with_message(err.bright().red().to_string());
            }
        },
    }
}
