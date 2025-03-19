use std::{collections::BTreeMap, fs};

use anyhow::{Context, Result, anyhow};
use ferrallay::mgmt::CACHE_DIR;
use walkdir::WalkDir;
use yansi::Paint;

use crate::cli::CacheSubcommand::{self, *};


pub fn process(subcommand: CacheSubcommand) {
    match subcommand {
        Info => info(),
        Clear => clear(),
    }
}

fn info() {
    println!("\nLocation: {}", CACHE_DIR.display().bright_blue().bold());

    let mut totals = BTreeMap::new();
    let files = WalkDir::new(&*CACHE_DIR)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file());
    for file in files {
        let sub = { file.path().components().rev() }
            .nth(file.depth() - 1)
            .expect("depth is smaller than path")
            .as_os_str()
            .to_owned();
        let size = file.metadata().map(|m| m.len()).unwrap_or_default();
        totals
            .entry(sub)
            .and_modify(|(count, total)| {
                *count += 1;
                *total += size;
            })
            .or_insert((1usize, size));
    }

    let mut total_count = 0;
    let mut total_size = 0;
    for (sub, (count, size)) in totals {
        total_count += count;
        total_size += size;
        println!(
            "\
--- {} ---
Files: {}
 Size: {}",
            sub.to_string_lossy().yellow(),
            count.bright_green().bold(),
            ::size::Size::from_bytes(size).cyan().bold(),
        );
    }

    println!(
        "\
--- Total ---
Files: {}
 Size: {}",
        total_count.bright_green().bold(),
        ::size::Size::from_bytes(total_size).cyan().bold(),
    );
}

fn clear() {
    if CACHE_DIR.exists() {
        for entry in WalkDir::new(&*CACHE_DIR).min_depth(1) {
            let res = match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if entry.file_type().is_dir() {
                        fs::remove_dir_all(path)
                    } else {
                        fs::remove_file(path)
                    }
                    .with_context(|| format!("Failed to delete cache location: {}", path.display().bold()))
                },
                Err(e) => Err(anyhow!(e)),
            };
            if let Err(e) = res {
                eprintln!("{:?}", e.red().wrap());
            }
        }
        println!("Cache cleared");
    }
}
