use std::{collections::HashMap, convert::identity, ops::Deref};

use anyhow::Result;
use colored::Colorize;
use once_cell::sync::Lazy;
use relibium::{
    client::schema::{AsProjectId, Author, ProjectId},
    config::Profile,
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
    eprint!("Querying metadata... ");

    let data = profile.data().await?;
    let mut projects = client
        .get_mods(data.mods.iter().map(|m| &m.id).collect::<Vec<_>>())
        .await?;

    // NOTE: Should these be moved into relibium?
    // Doing so would require making additional requests on each mod lookup even if the data isn't used

    // Load the Modrinth team members if we have a modrinth client
    if let Some(mr_client) = client.as_modrinth() {
        // Map the team ids to their index in the projects list
        let teams = projects
            .iter_mut()
            .enumerate()
            .fold(HashMap::new(), |mut acc, (i, m)| {
                if let ProjectId::Modrinth(_) = m.id {
                    for team in m.authors.drain(..) {
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
                .await?
                .into_iter()
                .flat_map(identity)
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
            if let Ok((own, repo)) = proj.id.try_as_github() {
                let mut page = gh_client
                    .repos(own, repo)
                    .releases()
                    .list()
                    .per_page(100)
                    .send()
                    .await?;
                loop {
                    let sum: u64 = page
                        .items
                        .into_iter()
                        .flat_map(|i| i.assets)
                        .map(|a| a.download_count as u64)
                        .sum();
                    proj.downloads += sum;
                    if let Some(p) = gh_client.get_page(&page.next).await? {
                        page = p;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    projects.sort_by_cached_key(|p| p.name.trim().to_lowercase());
    let projects = projects;

    let print = if markdown {
        print_project_markdown
    } else {
        print_project_verbose
    };

    projects.iter().map(Deref::deref).for_each(print);

    Ok(())
}

/*
pub fn curseforge_md(project: &Mod) {
    println!(
        "
**[{}]({})**
_{}_

|             |                 |
|-------------|-----------------|
| Source      | CurseForge `{}` |
| Open Source | {}              |
| Authors     | {}              |
| Categories  | {}              |",
        project.name.trim(),
        project.links.website_url,
        project.summary.trim(),
        project.id,
        project
            .links
            .source_url
            .as_ref()
            .map_or("No".into(), |url| format!("[Yes]({url})")),
        project
            .authors
            .iter()
            .map(|author| format!("[{}]({})", author.name, author.url))
            .format(", "),
        project
            .categories
            .iter()
            .map(|category| &category.name)
            .format(", "),
    );
}

pub fn modrinth_md(project: &Project, team_members: &[TeamMember]) {
    println!(
        "
**[{}](https://modrinth.com/mod/{})**
_{}_

|             |               |
|-------------|---------------|
| Source      | Modrinth `{}` |
| Open Source | {}            |
| Author      | {}            |
| Categories  | {}            |",
        project.title.trim(),
        project.id,
        project.description.trim(),
        project.id,
        project
            .source_url
            .as_ref()
            .map_or("No".into(), |url| { format!("[Yes]({url})") }),
        team_members
            .iter()
            .map(|member| format!(
                "[{}](https://modrinth.com/user/{})",
                member.user.username, member.user.id
            ))
            .format(", "),
        project.categories.iter().format(", "),
    );
}

pub fn github_md(repo: &Repository) {
    println!(
        "
**[{}]({})**{}

|             |             |
|-------------|-------------|
| Source      | GitHub `{}` |
| Open Source | Yes         |
| Owner       | [{}]({})    |{}",
        repo.name,
        repo.html_url.as_ref().unwrap(),
        repo.description
            .as_ref()
            .map_or(String::new(), |description| {
                format!("  \n_{}_", description.trim())
            }),
        repo.full_name.as_ref().unwrap(),
        repo.owner.as_ref().unwrap().login,
        repo.owner.as_ref().unwrap().html_url,
        repo.topics.as_ref().map_or(String::new(), |topics| format!(
            "\n| Topics | {} |",
            topics.iter().format(", ")
        )),
    );
}
 */
