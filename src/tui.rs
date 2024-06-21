use std::{borrow::Cow, fmt::Display, ops::Range};

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
        .template("{spinner} {bytes_per_sec} [{wide_bar:.cyan/blue}] {bytes:.cyan}/{total_bytes:.blue}")
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});

macro_rules! min {
    ($a:expr, $b:expr) => {
        if $a < $b {
            $a
        } else {
            $b
        }
    };
}
macro_rules! max {
    ($a:expr, $b:expr) => {
        if $a > $b {
            $a
        } else {
            $b
        }
    };
}
const fn ellipsis_mid(len: usize, max: usize) -> Range<usize> {
    if len <= max {
        return 0..0;
    }
    let bound = max / 2;
    let start = min!(bound, len);
    let end = min!(len, max!(bound, len.saturating_sub(bound - ((max + 1) & 1))));
    start..end
}
macro_rules! ellipsize {
    // Ellipsis middle
    (^ $str:ident, $max:expr) => {{
        let r = ellipsis_mid($str.len(), $max);
        format_args!("{}{}{}", &$str[..r.start], if r.is_empty() { "" } else { "…" }, &$str[r.end..],)
    }};
    // Ellipsis left
    (< $str:ident, $max:expr) => {{
        let mut i = $str.len().saturating_sub($max);
        format_args!("{}{}", if i > 0 { i += 1; "…" } else { "" }, &$str[i..])
    }};
    // Ellipsis right
    (> $str:ident, $max:expr) => {{
        let mut i = min!($str.len(), $max);
        format_args!("{1}{0}", if i < $str.len() { i -= 1; "…" } else { "" }, &$str[..i])
    }};
}
pub(crate) use ellipsize;

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
    let (game_version, loader, mods, pack) = profile.data().await.ok().map_or(
        (
            Cow::Borrowed(&*CROSS_RED),
            Cow::Borrowed(&*CROSS_RED),
            Cow::Borrowed(&*CROSS_RED),
            Cow::Borrowed(&*CROSS_RED),
        ),
        |data| {
            (
                Cow::Owned(data.game_version.green()),
                Cow::Owned(format!("{:?}", data.loader).purple()),
                Cow::Owned(data.mods.len().to_string().yellow()),
                Cow::Owned(
                    data.modpack
                        .as_deref()
                        .map_or_else(|| CROSS_RED.to_string(), mod_single_line)
                        .into(),
                ),
            )
        },
    );
    println!(
        "\
{}
    Path:        {}
    MC Version:  {}
    Mod Loader:  {}
    Mods:        {}
    Modpack:     {}
",
        {
            let mut name = profile.name().bold();
            if active {
                name = name.underline().italic();
            }
            name
        },
        profile.path().display().to_string().blue().underline(),
        game_version,
        loader,
        mods,
        pack,
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
        proj.website.as_ref().map(|u| u.as_str()).unwrap_or_default().blue().underline(),
        id_tag(&proj.id).dimmed(),
        proj.source_url
            .as_ref()
            .map(|u| u.as_str())
            .map(|u| format!("{} {}", *TICK_GREEN, u.blue().underline()).into())
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(&*CROSS_RED)),
        proj.downloads.to_string().yellow(),
        proj.authors.iter().format_with(", ", |a, fmt| fmt(&a.name.cyan())),
        proj.categories.iter().format_with(", ", |c, fmt| fmt(&c.magenta())),
        proj.license.as_ref().map_or_else(
            || "???".to_owned(),
            |l| {
                format!(
                    "{}{}",
                    l.spdx_id,
                    l.url
                        .as_ref()
                        .map_or_else(String::new, |url| format!(" ({})", url.as_str().blue().underline()))
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
        proj.website.as_ref().map(|u| u.as_str()).unwrap_or_default(),
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

pub fn fmt_profile_simple(p: &Profile, max_width: usize) -> String {
    let name = p.name();
    let path = p.path().display().to_string();
    let (name_width, path_width) = prop_widths(name.len(), path.len(), max_width);
    format!("{} • {}", ellipsize!(^name, name_width), ellipsize!(^path, path_width),)
}

const fn prop_widths(a: usize, b: usize, max: usize) -> (usize, usize) {
    let total = a + b;
    if total <= max {
        return (a, b);
    }

    let (short, long) = if a <= b { (a, b) } else { (b, a) };

    let over_total = total - max;
    let over_long = long.saturating_sub(short * 2);
    let long = long - min!(over_long, over_total);

    let total = short + long;
    let short = max * short / total;
    let long = (max * long).div_ceil(total);

    if a < b {
        (short, long)
    } else {
        (long, short)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STR: &str = "12345678909876543210";

    #[test]
    fn ellipsize_mid() {
        // Smaller
        assert_eq!("12…10", ellipsize!(^ TEST_STR, 5).to_string());
        // Equal
        assert_eq!(TEST_STR, ellipsize!(^ TEST_STR, TEST_STR.len()).to_string());
        // Larger
        assert_eq!(TEST_STR, ellipsize!(^ TEST_STR, TEST_STR.len() * 2).to_string());
    }

    #[test]
    fn ellipsize_start() {
        // Smaller
        assert_eq!("…3210", ellipsize!(< TEST_STR, 5).to_string());
        // Equal
        assert_eq!(TEST_STR, ellipsize!(< TEST_STR, TEST_STR.len()).to_string());
        // Larger
        assert_eq!(TEST_STR, ellipsize!(< TEST_STR, TEST_STR.len() * 2).to_string());
    }

    #[test]
    fn ellipsize_end() {
        // Smaller
        assert_eq!("1234…", ellipsize!(> TEST_STR, 5).to_string());
        // Equal
        assert_eq!(TEST_STR, ellipsize!(> TEST_STR, TEST_STR.len()).to_string());
        // Larger
        assert_eq!(TEST_STR, ellipsize!(> TEST_STR, TEST_STR.len() * 2).to_string());
    }
}
