#![deny(missing_docs)]

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use relibium::{config::ModLoader, DEFAULT_MINECRAFT_DIR};

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(arg_required_else_help = true)]
pub struct Ferium {
    #[clap(subcommand)]
    pub subcommand: SubCommands,
    /// Sets the number of worker threads the tokio runtime will use.
    /// You can also use the environment variable `TOKIO_WORKER_THREADS`.
    #[clap(long, short)]
    pub threads: Option<usize>,
    /// Set a GitHub personal access token for increasing the GitHub API rate
    /// limit. You can also use the environment variable `GITHUB_TOKEN`.
    #[clap(long, visible_alias = "gh")]
    pub github_token: Option<String>,
    /// Set a custom CurseForge API key.
    /// You can also use the environment variable `CURSEFORGE_API_KEY`.
    #[clap(long, visible_alias = "cf")]
    pub curseforge_api_key: Option<String>,
    /// Set the file to read the config from.
    /// This does not change the `cache` and `tmp` directories.
    /// You can also use the environment variable `FERIUM_CONFIG_FILE`.
    #[clap(long, short, visible_aliases = ["config", "conf"])]
    #[clap(value_hint(ValueHint::FilePath))]
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    /// Add mods to the active profile
    #[clap(visible_aliases = ["new", "create"])]
    Add {
        /// The identifier(s) of the mod/project/repository
        ///
        /// The Modrinth project ID is specified at the bottom of the left
        /// sidebar under 'Technical information'. You can also use the project
        /// slug in the URL. The CurseForge project ID is specified at the top
        /// of the right sidebar under 'About Project'. The GitHub identifier is
        /// the repository's full name, e.g. `gorilla-devs/ferium`.
        identifiers: Vec<String>,
    },
    /// Print shell auto completions for the specified shell
    Complete {
        /// The shell to generate auto completions for
        #[clap(value_enum)]
        shell: Shell,
    },
    /// List all the mods in the profile, and with their metadata if verbose
    #[clap(visible_alias = "mods")]
    List {
        /// Show additional information about the mod
        #[clap(long, short)]
        verbose: bool,
        /// Output information in markdown format and alphabetical order
        ///
        /// Useful for creating modpack mod lists.
        /// Complements the verbose flag.
        #[clap(long, short, visible_alias = "md")]
        markdown: bool,
    },
    /// Add, configure, delete, switch, list, or upgrade modpacks
    Modpack {
        #[clap(subcommand)]
        subcommand: Option<ModpackSubCommands>,
    },
    /// Create, configure, delete, switch, or list profiles
    Profile {
        #[clap(subcommand)]
        subcommand: Option<ProfileSubCommands>,
    },
    /// List all the profiles with their data
    Profiles,
    /// Remove mods and/or repositories from the profile.
    /// Optionally, provide a list of names or IDs of the mods to remove.
    #[clap(visible_aliases = ["rm", "delete", "del"])]
    Remove {
        /// List of project IDs or case-insensitive names of mods to remove
        mod_names: Vec<String>,
    },
    /// Download and install the latest compatible version of your mods
    #[clap(visible_aliases = ["download", "install"])]
    Upgrade,
}

#[derive(Subcommand)]
pub enum ProfileSubCommands {
    /// Show information about the current profile
    Info,
    /// List all the profiles with their data
    List,
    /// Create a new profile.
    /// Optionally, provide the settings as arguments.
    #[clap(visible_aliases = ["create"])]
    New {
        /// The Minecraft version to use
        #[clap(long, short = 'v')]
        game_version: Option<String>,
        /// The mod loader to use
        #[clap(long, short)]
        #[clap(value_enum)]
        loader: Option<ModLoader>,
        /// The name of the profile
        #[clap(long, short)]
        name: Option<String>,
        /// The directory to output mods to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        path: Option<PathBuf>,
    },
    /// Add/import and existing profile path to the config
    #[clap(visible_aliases = ["import"])]
    Add {
        /// The name of the profile
        #[clap(long, short)]
        name: String,
        /// The directory containing an existing profile config file
        #[clap(value_hint(ValueHint::DirPath), default_value = DEFAULT_MINECRAFT_DIR.as_os_str())]
        path: PathBuf,
    },
    /// Delete a profile.
    /// Optionally, provide the name of the profile to delete.
    #[clap(visible_aliases = ["rm", "delete", "del"])]
    Remove {
        /// The name of the profile to delete
        profile_name: Option<String>,
        /// The name of the profile to switch to afterwards
        #[clap(long, short)]
        switch_to: Option<String>,
    },
    /// Configure the current profile's name, Minecraft version, mod loader, and
    /// output directory. Optionally, provide the settings to change as
    /// arguments.
    #[clap(visible_aliases = ["config", "conf"])]
    Configure {
        /// The Minecraft version to use
        #[clap(long, short = 'v')]
        game_version: Option<String>,
        /// The mod loader to use
        #[clap(long, short)]
        #[clap(value_enum)]
        loader: Option<ModLoader>,
        /// The name of the profile
        #[clap(long, short)]
        name: Option<String>,
    },
    /// Switch between different profiles.
    /// Optionally, provide the name of the profile to switch to.
    Switch {
        /// The name of the profile to switch to
        profile_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ModpackSubCommands {
    /// Show information about the current modpack
    Info,
    /// Set a modpack on the active profile.
    #[clap(visible_aliases = ["new", "create"])]
    Add {
        /// The identifier of the modpack/project
        ///
        /// The Modrinth project ID is specified at the bottom of the left
        /// sidebar under 'Technical information'. You can also use the
        /// project slug for this. The CurseForge project ID is
        /// specified at the top of the right sidebar under 'About Project'.
        id: String,
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        #[clap(long, short)]
        install_overrides: Option<bool>,
    },
    /// Delete modpack from the active profile.
    #[clap(visible_aliases = ["rm", "delete", "del"])]
    Remove {
        /// Delete without a confirmation prompt
        #[clap(long, short)]
        force: bool,
    },
    /// Configure the current modpack's output directory and installation of
    /// overrides. Optionally, provide the settings to change as arguments.
    #[clap(visible_aliases = ["config", "conf"])]
    Configure {
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        #[clap(long, short)]
        install_overrides: Option<bool>,
    },
}
