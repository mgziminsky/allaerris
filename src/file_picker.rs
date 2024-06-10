use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Result;

static HOME: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::home_dir().expect("should be able to determine home dir"));

#[cfg(feature = "gui")]
/// Use the system file picker to pick a file, with a `default` path (that is [not supported on linux](https://github.com/PolyMeilex/rfd/issues/42))
fn show_folder_picker(default: &Path, prompt: impl Into<String>) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_can_create_directories(true)
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
}

#[cfg(any(not(feature = "gui"), ide))]
/// Use a terminal input to pick a file, with a `default` path
fn show_folder_picker(default: &Path, prompt: impl Into<String>) -> Option<PathBuf> {
    dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .default(default.display().to_string())
        .with_prompt(prompt)
        .report(false)
        .interact()
        .map(Into::into)
        .ok()
}

/// Pick a folder using the terminal or system file picker (depending on the features flag `gui`)
pub fn pick_folder(default: impl AsRef<Path>, prompt: &str) -> Result<PathBuf> {
    let default = default.as_ref();
    let path = show_folder_picker(default, prompt).unwrap_or_else(|| default.to_owned());

    // Replace a leading ~ with the home dir
    let mut parts = path.components();
    let path = parts
        .next()
        .map(|part| {
            if part.as_os_str() == "~" {
                HOME.as_path()
            } else {
                part.as_os_str().as_ref()
            }
        })
        .map_or_else(PathBuf::new, |p| p.join(parts.as_path()));

    Ok(path)
}
