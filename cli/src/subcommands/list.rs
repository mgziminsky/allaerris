use std::{collections::HashMap, ops::Deref};

use anyhow::{Context, Result};
use colored::Colorize;
use once_cell::sync::Lazy;
use relibium::{
    client::schema::{Author, ProjectId, ProjectIdSvcType},
    config::{Mod, Profile},
    modrinth::apis::teams_api::GetTeamsParams,
    Client,
};

use crate::tui::{print_mods, print_project_markdown, print_project_verbose};

static MR_BASE: Lazy<::url::Url> = Lazy::new(|| {
    "https://modrinth.com/user/"
        .parse()
        .expect("base url should always parse successfully")
});

pub async fn simple(profile: &Profile) -> Result<(), anyhow::Error> {
    let data = profile.data().await?;
    print_mods(
        format_args!(
            "{} {} on {} {}",
            profile.name().bold(),
            format!("({} mods)", data.mods.len()).yellow(),
            format!("{:?}", data.loader).purple(),
            data.game_version.green(),
        ),
        &data.mods,
    );
    Ok(())
}

pub async fn verbose(client: &Client, profile: &Profile, markdown: bool) -> Result<()> {
    eprintln!("{}\n", "Querying metadata...".dimmed());

    let data = profile.data().await?;
    let mut projects = client
        .get_mods(&data.mods.iter().map(Mod::id).map(|id| id as _).collect::<Vec<_>>())
        .await
        .context("Failed to load mod details")?;

    // Load the Modrinth team members if we have a modrinth client
    if let Some(mr_client) = client.as_modrinth() {
        // Map the team ids to their index in the projects list
        let teams = projects.iter_mut().enumerate().fold(HashMap::new(), |mut acc, (i, m)| {
            if let ProjectId::Modrinth(_) = m.id {
                for team in m.authors.drain(..) {
                    #[allow(clippy::unwrap_or_default)]
                    acc.entry(team.name).or_insert_with(Vec::new).push(i);
                }
            }
            acc
        });
        if !teams.is_empty() {
            let team_ids = teams.keys().map(AsRef::as_ref).collect::<Vec<_>>();
            // Get the members for all the teams
            mr_client
                .teams()
                .get_teams(&GetTeamsParams { ids: &team_ids })
                .await
                .context("Failed to load Modrinth teams")?
                .into_iter()
                .flatten()
                // Add member names to the author list of every mod they belong to
                .for_each(|member| {
                    if let Some(indices) = teams.get(&member.team_id) {
                        // Clone into all but the last project which can take ownership
                        if let Some((last, front)) = indices.split_last() {
                            let author = Author {
                                name: member.user.name.unwrap_or(member.user.username),
                                url: MR_BASE.join(&member.user.id).ok(),
                            };
                            for v in front {
                                projects[*v].authors.push(author.clone());
                            }
                            projects[*last].authors.push(author);
                        }
                    }
                });
        }
    }

    // Calculate Github downloads count from releases
    // FIXME: Rate Limiting
    if let Some(gh_client) = client.as_github() {
        for proj in projects.iter_mut() {
            if let Ok((own, repo)) = proj.id.get_github() {
                let mut page = gh_client.repos(own, repo).releases().list().per_page(100).send().await?;
                loop {
                    let sum: u64 = page.items.into_iter().flat_map(|i| i.assets).map(|a| a.download_count as u64).sum();
                    proj.downloads += sum;
                    let Some(p) = gh_client.get_page(&page.next).await? else {
                        break;
                    };
                    page = p;
                }
            }
        }
    }

    projects.sort_by_cached_key(|p| p.name.trim().to_lowercase());
    let projects = projects;

    let print = if markdown { print_project_markdown } else { print_project_verbose };

    projects.iter().map(Deref::deref).for_each(print);

    Ok(())
}
