use anyhow::Result;
use dialoguer::{Input, Select};
use relibium::config::{ModLoader, Profile};

use super::{pick_minecraft_version, pick_mod_loader};
use crate::tui::THEME;

pub async fn configure(
    profile: &mut Profile,
    game_version: Option<String>,
    loader: Option<ModLoader>,
    name: Option<String>,
) -> Result<()> {
    let mut interactive = true;

    {
        let data = profile.data_mut().await?;
        if let Some(game_version) = game_version {
            data.game_version = game_version;
            interactive = false;
        }
        if let Some(loader) = loader {
            data.loader = loader;
            interactive = false;
        }
    }
    if let Some(name) = name {
        profile.set_name(name);
        interactive = false;
    }
    if interactive {
        let items = vec![
            // Show a picker of Minecraft versions to select from
            "Minecraft version",
            // Show a picker to change mod loader
            "Mod loader",
            // Show a dialog to change name
            "Profile Name",
            // Quit the configuration
            "Quit",
        ];

        loop {
            let selection = Select::with_theme(&*THEME)
                .with_prompt("Which setting would you like to change")
                .items(&items)
                .interact_opt()?;

            if let Some(index) = selection {
                let data = profile.data_mut().await?;
                match index {
                    0 => data.game_version = pick_minecraft_version().await?,
                    1 => data.loader = pick_mod_loader(Some(&data.loader))?,
                    2 => {
                        let name = Input::with_theme(&*THEME)
                            .with_prompt("Change the profile's name")
                            .default(profile.name().to_owned())
                            .interact_text()?;
                        profile.set_name(name);
                    }
                    3 => break,
                    _ => unreachable!(),
                }
                println!();
            } else {
                break;
            }
        }
    }

    Ok(())
}
