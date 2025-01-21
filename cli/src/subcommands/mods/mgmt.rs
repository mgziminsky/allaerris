use std::{collections::HashMap, sync::mpsc};

use anyhow::{bail, Result};
use dialoguer::MultiSelect;
use ferrallay::{
    client::schema::ProjectType,
    config::{Mod, ModLoader, Profile, ProjectWithVersion},
    mgmt::{
        events::{DownloadId, DownloadProgress, ProgressEvent},
        server::Version,
    },
    Client, ProfileManager,
};
use indicatif::{MultiProgress, ProgressBar};
use yansi::Paint;

use crate::{
    cli::{MgmtCommand, ServerSubcommand},
    consts,
    helpers::path_profile,
    tui::{const_style, ellipsize, id_tag, CROSS_RED, PROG_BYTES, PROG_DONE, THEME, TICK_GREEN, TICK_YELLOW},
};


pub async fn process(command: MgmtCommand, client: &Client, profile: &mut Profile) -> Result<()> {
    use MgmtCommand::*;
    let (sender, handle) = progress_hander();
    {
        let mut manager = ProfileManager::with_channel(sender);
        match command {
            Apply { force, no_cache } => {
                manager.force = force;
                manager.no_cache = no_cache;
                manager.apply(client, profile).await?;
            },
            Update { ids, revert, apply } => {
                assert!(!(revert && apply), "Revert and Apply should never both be set");
                update(&manager, profile, client, ids, revert).await?;
                if apply || (!revert && prompt_apply()) {
                    manager.apply(client, profile).await?;
                } else if !revert {
                    println!(
                        "\n{}",
                        format_args!(
                            "Updates have not yet been installed! To install, run `{}`",
                            const_style!(concat!(consts!(APP_NAME), " apply"); bold())
                        )
                        .yellow()
                        .wrap()
                    );
                }
            },
            Scan { all, lock } => {
                scan_all(&manager, client, profile, all, lock).await?;
            },
            Server(subcommand) => match subcommand {
                ServerSubcommand::Install {
                    out,
                    loader,
                    minecraft,
                    version,
                    no_cache,
                } => {
                    manager.no_cache = no_cache;
                    let out = out.as_deref().unwrap_or(profile.path());
                    let (loader, minecraft) = match path_profile(Some(out)) {
                        Some(mut profile) if profile.data().await.is_ok() => {
                            let data = profile.data_mut().await.unwrap();
                            (
                                loader.or(data.loader.known()),
                                minecraft.or(Some(std::mem::take(&mut data.game_version))),
                            )
                        },
                        _ => (loader, minecraft),
                    };
                    let Some(version) = version.as_deref().map(Version::Exact).or(minecraft.as_deref().map(Version::Latest)) else {
                        bail!("Missing server version");
                    };
                    let loader = loader.and_then(ModLoader::known);
                    let server = manager.server_install(loader, version, out).await?;
                    println!("Server successfully installed at `{}`", server.display().green());
                },
            },
        }
    }
    let _ = handle.await;
    Ok(())
}

async fn update(manager: &ProfileManager, profile: &Profile, client: &Client, ids: Vec<String>, revert: bool) -> Result<()> {
    let updates = if revert {
        manager.revert(profile).await?
    } else {
        let ids = ids.iter().map(|id| id as _).collect::<Vec<_>>();
        manager.update(client, profile, &ids).await?
    };
    if updates.is_empty() {
        println!("Profile is up to date");
    } else {
        let (tick, label) = if revert { (TICK_YELLOW, "Reverted") } else { (TICK_GREEN, "Updated") };
        for up in updates {
            println!(
                "{tick} {label} {} from version {} -> {}\n\t{} -> {}",
                id_tag(&up.project).bold().wrap(),
                up.from.0.bold().yellow(),
                up.to.0.bold().blue(),
                up.from.1.display().bold().yellow(),
                up.to.1.display().bold().blue(),
            );
        }
    }
    Ok(())
}

async fn scan_all(manager: &ProfileManager, client: &Client, profile: &mut Profile, all: bool, lock: bool) -> Result<()> {
    const TYPES: [ProjectType; 4] = [
        ProjectType::Mod,
        ProjectType::ResourcePack,
        ProjectType::DataPack,
        ProjectType::Shader,
    ];
    let mut found = vec![];
    for typ in TYPES {
        found.extend(scan(manager, client, profile, typ, all, lock).await?);
    }
    let new = super::add::add_mods(profile.data_mut().await?, &found);
    if new > 0 {
        profile.save().await?;
    }
    Ok(())
}

async fn scan(
    manager: &ProfileManager,
    client: &Client,
    profile: &mut Profile,
    typ: ProjectType,
    all: bool,
    lock: bool,
) -> Result<Vec<Mod>> {
    let found = manager.scan(client, profile, typ, all).await?.into_iter().collect::<Vec<_>>();
    if found.is_empty() {
        println!("All `{}` files are already present in profile", typ.install_dir());
        return Ok(vec![]);
    }
    let selection = MultiSelect::new()
        .with_prompt("Select which mods to add to profile")
        .report(false)
        .items_checked(
            &found
                .iter()
                .map(|(p, v)| {
                    (
                        format!(
                            "[{}] {:50} => {:50}",
                            id_tag(&v.project_id),
                            ellipsize!(< p.display().to_string(), 50).bright_blue(),
                            ellipsize!(^ &v.title, 50).bold().cyan(),
                        ),
                        true,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .interact_opt()?
        .unwrap_or_default();
    Ok({
        let mut found = found;
        selection
            .into_iter()
            .rev()
            .map(|i| found.swap_remove(i).1)
            .map(|v| Mod {
                id: ProjectWithVersion::new(v.project_id, lock.then_some(v.id)).unwrap(),
                slug: String::new(),
                name: format!("[SCANNED] {}", v.title),
                exclude: false,
                project_type: typ,
            })
            .collect::<Vec<_>>()
    })
}

fn prompt_apply() -> bool {
    dialoguer::Confirm::with_theme(&*THEME)
        .with_prompt("Install updates now?")
        .default(false)
        .interact()
        .unwrap_or(false)
}

fn progress_hander() -> (mpsc::Sender<ProgressEvent>, tokio::task::JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let handle = tokio::task::spawn_blocking(move || {
        let progress = MultiProgress::new();
        let mut bars = HashMap::new();
        while let Ok(evt) = receiver.recv() {
            use ProgressEvent::*;
            match evt {
                Status(msg) => {
                    println!("{msg}");
                },
                Download(evt) => handle_dl(evt, &mut bars, &progress),
                Installed { file, is_new, typ } => {
                    use ferrallay::mgmt::events::InstallType::*;
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
                Deleted(file) => {
                    println!("{}   Deleted: {}", TICK_GREEN, file.display());
                },
                Error(err) => {
                    eprintln!("{}", err.red());
                },
            }
        }
    });
    (sender, handle)
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
