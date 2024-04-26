use std::{borrow::Cow, fmt::Display};

use colored::{ColoredString, Colorize};
use dialoguer::theme::ColorfulTheme;
use indicatif::ProgressStyle;
use once_cell::sync::Lazy;
use relibium::{
    client::schema::ProjectId,
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

pub fn mod_single_line(m: &Mod) -> String {
    let id = match &m.id {
        ProjectId::Forge(id) => format!("{} {id:8}", *CF),
        ProjectId::Modrinth(id) => format!("{} {id:8}", *MR),
        ProjectId::Github(_) => GH.to_string(),
    };
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
        \r  Path:               {}
        \r  Minecraft Version:  {}
        \r  Mod Loader:         {}
        \r  Mods:               {}\n",
        profile.name().bold(),
        if active { " *" } else { "" },
        profile.path().display().to_string().blue().underline(),
        game_version,
        loader,
        mods,
    );
}
