use std::fs::{self};

use anyhow::{anyhow, Context, Result};
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
    let mut count = 0usize;
    let mut size = 0;
    let files = WalkDir::new(&*CACHE_DIR)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file());
    for file in files {
        count += 1;
        if let Ok(meta) = file.metadata() {
            size += meta.len();
        }
    }

    println!(
        "
  Location: {}
File Count: {}
Total Size: {}
",
        CACHE_DIR.display().bright_blue().bold(),
        count.bright_green().bold(),
        ::size::Size::from_bytes(size).cyan().bold(),
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
