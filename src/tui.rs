use std::{borrow::Cow, fmt::Display};

use colored::{ColoredString, Colorize};
use dialoguer::theme::ColorfulTheme;
use indicatif::ProgressStyle;
use itertools::Itertools;
use once_cell::sync::Lazy;
use relibium::{
    client::schema::{Project, ProjectId},
    config::{Mod, Profile},
};

static CF: Lazy<ColoredString> = Lazy::new(|| "CF".red());
static MR: Lazy<ColoredString> = Lazy::new(|| "MR".green());
static GH: Lazy<ColoredString> = Lazy::new(|| "GH".purple());

pub const CROSS: &str = "🗙";
pub static CROSS_RED: Lazy<ColoredString> = Lazy::new(|| CROSS.red());

pub const TICK: &str = "✓";
pub static TICK_GREEN: Lazy<ColoredString> = Lazy::new(|| TICK.green());
pub static TICK_YELLOW: Lazy<ColoredString> = Lazy::new(|| TICK.yellow());

pub static THEME: Lazy<ColorfulTheme> = Lazy::new(Default::default);
pub static STYLE_NO: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::default_bar()
        .template("{spinner} {elapsed} [{wide_bar:.cyan/blue}] {pos:.cyan}/{len:.blue}")
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});
pub static STYLE_BYTE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::default_bar()
        .template(
            "{spinner} {bytes_per_sec} [{wide_bar:.cyan/blue}] {bytes:.cyan}/{total_bytes:.blue}",
        )
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});

fn id_tag(id: &ProjectId) -> String {
    match id {
        ProjectId::Forge(id) => format!("{} {id:8}", *CF),
        ProjectId::Modrinth(id) => format!("{} {id:8}", *MR),
        ProjectId::Github(_) => GH.to_string(),
    }
}

pub fn mod_single_line(m: &Mod) -> String {
    let id = id_tag(&m.id);
    let name = match &m.id {
        ProjectId::Forge(_) | ProjectId::Modrinth(_) => m.name.bold().to_string(),
        ProjectId::Github((owner, repo)) => format!("{}/{}", owner.dimmed(), repo.bold()),
    };
    format!("{id} ― {name}")
}

pub fn print_mods(label: impl Display, mods: &[Mod]) {
    println!("{label}");
    if let Some((last, rest)) = mods.split_last() {
        for m in rest {
            println!("  ├─{}", mod_single_line(m));
        }
        println!("  └─{}", mod_single_line(last));
    }
}

pub async fn print_profile(profile: &Profile, active: bool) {
    let (game_version, loader, mods) = profile.data().await.ok().map_or(
        (
            Cow::Borrowed(&*CROSS_RED),
            Cow::Borrowed(&*CROSS_RED),
            Cow::Borrowed(&*CROSS_RED),
        ),
        |data| {
            (
                Cow::Owned(data.game_version.green()),
                Cow::Owned(format!("{:?}", data.loader).purple()),
                Cow::Owned(data.mods.len().to_string().yellow()),
            )
        },
    );
    println!(
        "{}{}
  Path:               {}
  Minecraft Version:  {}
  Mod Loader:         {}
  Mods:               {}
",
        profile.name().bold(),
        if active { " *" } else { "" },
        profile.path().display().to_string().blue().underline(),
        game_version,
        loader,
        mods,
    );
}

pub fn print_project_verbose(proj: &Project) {
    println!(
        "\
{}
{}

  Link:\t\t{}
  Project ID:\t{}
  Open Source:\t{}
  Downloads:\t{}
  Authors:\t{}
  Categories:\t{}
  License:\t{}
",
        proj.name.trim().bold(),
        proj.description.trim().italic(),
        proj.website
            .as_ref()
            .map(|u| u.as_str())
            .unwrap_or_default()
            .blue()
            .underline(),
        id_tag(&proj.id).dimmed(),
        proj.source_url
            .as_ref()
            .map(|u| u.as_str())
            .map(|u| format!("{} {}", *TICK_GREEN, u.blue().underline()).into())
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(&*CROSS_RED)),
        proj.downloads.to_string().yellow(),
        proj.authors
            .iter()
            .format_with(", ", |a, fmt| fmt(&a.name.cyan())),
        proj.categories
            .iter()
            .format_with(", ", |c, fmt| fmt(&c.magenta())),
        proj.license.as_ref().map_or_else(
            || "???".to_owned(),
            |l| {
                format!(
                    "{}{}",
                    l.spdx_id,
                    l.url.as_ref().map_or_else(String::new, |url| format!(
                        " ({})",
                        url.as_str().blue().underline()
                    ))
                )
            }
        ),
    );
}

pub fn print_project_markdown(proj: &Project) {
    println!(
        "\
**[{}]({})**
_{}_

|             |    |
|-------------|----|
| Source      | {} |
| Open Source | {} |
| Authors     | {} |
| Categories  | {} |
",
        proj.name.trim(),
        proj.website
            .as_ref()
            .map(|u| u.as_str())
            .unwrap_or_default(),
        proj.description.trim(),
        format_args!(
            "{} `{}`",
            match &proj.id {
                ProjectId::Forge(_) => "Forge",
                ProjectId::Modrinth(_) => "Modrinth",
                ProjectId::Github(_) => "Github",
            },
            proj.id
        ),
        proj.source_url
            .as_ref()
            .map(|u| u.as_str())
            .map(|u| format!("[YES]({u})").into())
            .unwrap_or(Cow::Borrowed("NO")),
        proj.authors.iter().format_with(", ", |a, fmt| {
            if let Some(url) = a.url.as_ref() {
                fmt(&format_args!("[{}]({})", a.name, url))
            } else {
                fmt(&a.name)
            }
        }),
        proj.categories.join(", "),
    );
}
