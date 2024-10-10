use std::{borrow::Cow, fmt::Display, ops::Range, sync::LazyLock};

use anyhow::anyhow;
use dialoguer::theme::ColorfulTheme;
use ferrallay::{
    client::schema::{Project, ProjectId, VersionId},
    config::{Mod, Profile, VersionedProject},
};
use indicatif::ProgressStyle;
use itertools::Itertools;
use yansi::{Paint, Painted};


/// Creates a compile time `const` styled value
macro_rules! const_style {
    ($val:expr; $($mods:tt)+) => {
        const {
            #[allow(unused_imports)]
            use ::yansi::Paint;
            ::yansi::Painted::new($val).$($mods)+
        }
    };
}
pub(crate) use const_style;


const CF: Painted<&str> = Painted::new("CF").red();
const MR: Painted<&str> = Painted::new("MR").green();
const GH: Painted<&str> = Painted::new("GH").magenta();

pub const CROSS: &str = "✗";
pub const CROSS_RED: Painted<&str> = Painted::new(CROSS).red();

pub const TICK: &str = "✓";
pub const TICK_GREEN: Painted<&str> = Painted::new(TICK).green();
pub const TICK_YELLOW: Painted<&str> = Painted::new(TICK).yellow();

pub static THEME: LazyLock<ColorfulTheme> = LazyLock::new(Default::default);
pub static PROG_BYTES: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template(
        "{spinner} {msg:50!} {eta:>3.bold.yellow} {wide_bar:.cyan/blue} [{bytes_per_sec:.green} | {bytes:.cyan} / {total_bytes:.blue}]",
    )
    .expect("template should be valid")
});
pub static PROG_DONE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::with_template("{prefix:.bold} {bytes:>10.cyan} {elapsed:>3.yellow} {msg}").expect("template should be valid")
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
pub(crate) const fn ellipsis_mid(len: usize, max: usize) -> Range<usize> {
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
    (^ $str:expr, $max:expr) => {{
        let value = $str;
        let r = crate::tui::ellipsis_mid(value.len(), $max);
        format!("{}{}{}", &value[..r.start], if r.is_empty() { "" } else { "…" }, &value[r.end..],)
    }};
    // Ellipsis left
    (< $str:expr, $max:expr) => {{
        let value = $str;
        let mut i = value.len().saturating_sub($max);
        format!(
            "{}{}",
            if i > 0 {
                i += 1;
                "…"
            } else {
                ""
            },
            &value[i..]
        )
    }};
    // Ellipsis right
    (> $str:expr, $max:expr) => {{
        let value = $str;
        let mut i = min!(value.len(), $max);
        format!(
            "{1}{0}",
            if i < value.len() {
                i -= 1;
                "…"
            } else {
                ""
            },
            &value[..i]
        )
    }};
}
pub(crate) use ellipsize;

pub fn id_tag(id: &ProjectId) -> String {
    match id {
        ProjectId::Forge(id) => format!("{CF} {id}"),
        ProjectId::Modrinth(id) => format!("{MR} {id}"),
        ProjectId::Github((ref own, ref repo)) => format!("{GH} {own}/{repo}"),
    }
}
pub fn vid_tag(id: &VersionId) -> String {
    match id {
        VersionId::Forge(id) => format!("{CF} {id}"),
        VersionId::Modrinth(id) => format!("{MR} {id}"),
        VersionId::Github(id) => format!("{GH} {id}"),
    }
}

pub fn mod_single_line(m: &Mod) -> String {
    let id = id_tag(m.project());
    let name = match m.project() {
        ProjectId::Forge(_) | ProjectId::Modrinth(_) => m.name.bold().to_string(),
        ProjectId::Github((owner, repo)) => format!("{}/{}", owner.dim(), repo.bold()),
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
    let data = profile.data().await;
    println!(
        "\
{}{}
    Path:        {}
    {}
",
        active.then_some("*").unwrap_or_default(),
        {
            let mut name = profile.name().bold();
            if active {
                name = name.underline().italic();
            }
            name
        },
        profile.path().display().bright_blue().underline(),
        data.map_or_else(
            |e| format!("Error:       {:?}", anyhow!(e).red()),
            |d| format!(
                "\
    MC Version:  {}
    Mod Loader:  {}
    Mods:        {}
    Modpack:     {}",
                d.game_version.green(),
                format_args!("{:?}", d.loader).magenta(),
                d.mods.len().yellow(),
                d.modpack.as_deref().map_or_else(|| CROSS_RED.to_string(), mod_single_line)
            )
        ),
    );
}

pub fn print_project_verbose(proj: &Project) {
    println!(
        "\
{}
{}
    Link:         {}
    Project ID:   {}
    Open Source:  {}
    Downloads:    {}
    Authors:      {}
    Categories:   {}
    License:      {}
",
        proj.name.trim().bold(),
        proj.description.trim().italic(),
        proj.website
            .as_ref()
            .map(url::Url::as_str)
            .unwrap_or_default()
            .bright_blue()
            .underline(),
        id_tag(&proj.id),
        proj.source_url.as_ref().map(url::Url::as_str).map_or_else(
            || CROSS_RED.to_string(),
            |u| format!("{} {}", TICK_GREEN, u.bright_blue().underline())
        ),
        proj.downloads.yellow(),
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
                        .map_or_else(String::new, |url| format!(" ({})", url.bright_blue().underline()))
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
        proj.website.as_ref().map(url::Url::as_str).unwrap_or_default(),
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
            .map(url::Url::as_str)
            .map_or(Cow::Borrowed("NO"), |u| format!("[YES]({u})").into()),
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

// Compile time tests of the `prop_widths` fn
macro_rules! assert_widths {
    // Copied and adapted from static_assertions
    ($(($a:literal, $b:literal, $x:literal) == ($l:literal, $r:literal));*$(;)?) => {$(
        const _: [(); !matches!(prop_widths($a, $b, $x), ($l, $r)) as usize] = [];
    )*};
}
assert_widths! {
    (20, 20, 20) == (10, 10);
    (10, 20, 15) == ( 5, 10);
    (10, 50, 30) == (10, 20);
    (50, 10, 30) == (20, 10);
    (20, 25, 30) == (13, 17);
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
