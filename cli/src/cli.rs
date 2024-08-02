#![deny(missing_docs)]

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use relibium::{config::ModLoader, DEFAULT_MINECRAFT_DIR};

#[derive(Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help = true)]
pub struct Ferium {
    #[command(subcommand)]
    pub subcommand: SubCommand,
    /// Sets the number of worker threads the tokio runtime will use.
    /// You can also use the environment variable `TOKIO_WORKER_THREADS`.
    #[arg(long, short)]
    pub threads: Option<usize>,
    /// Set a GitHub personal access token for increasing the GitHub API rate
    /// limit. You can also use the environment variable `GITHUB_TOKEN`.
    #[arg(long, visible_alias = "gh")]
    pub github_token: Option<String>,
    /// Set a custom CurseForge API key.
    /// You can also use the environment variable `CURSEFORGE_API_KEY`.
    #[arg(long, visible_alias = "cf")]
    pub curseforge_api_key: Option<String>,
    /// Set the file to read the config from.
    /// This does not change the `cache` and `tmp` directories.
    /// You can also use the environment variable `FERIUM_CONFIG_FILE`.
    #[arg(long, short, visible_aliases = ["config", "conf"])]
    #[arg(value_hint(ValueHint::FilePath))]
    pub config_file: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum SubCommand {
    #[command(flatten)]
    Mods(ModsSubcommand),

    /// Add, configure, or delete the current modpack
    Modpack {
        #[command(subcommand)]
        subcommand: Option<ModpackSubCommand>,
    },

    /// Create, configure, delete, switch, or list profiles
    Profile {
        #[command(subcommand)]
        subcommand: Option<ProfileSubCommand>,
    },

    /// List all the profiles with their data
    Profiles,

    /// Print shell auto completions for the specified shell
    Complete {
        /// The shell to generate auto completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum ModsSubcommand {
    /// Add mods to the active profile
    Add {
        /// The identifier(s) of the mod/project/repository
        ///
        /// The Modrinth project ID is specified at the bottom of the left
        /// sidebar under 'Technical information'. You can also use the project
        /// slug in the URL.
        /// The CurseForge project ID is specified at the top of the right
        /// sidebar under 'About Project'.
        /// The GitHub identifier is the repository's full name, e.g.
        /// `gorilla-devs/ferium`.
        ids: Vec<String>,

        /// Prevent the mod(s) from being installed
        ///
        /// This will prevent the specified mods from being installed as part of
        /// a modpack
        #[arg(short = 'x', long)]
        exclude: bool,
    },

    /// Remove mods and/or repositories from the profile.
    /// Optionally, provide a list of names or IDs of the mods to remove.
    #[command(visible_aliases = ["rm", "delete", "del"])]
    Remove {
        /// List of project IDs or case-insensitive names of mods to remove
        mod_names: Vec<String>,
    },

    /// List all the mods in the profile, including their metadata if verbose
    #[command(visible_alias = "mods")]
    List {
        /// Show additional information about the mod
        #[arg(long, short)]
        verbose: bool,
        /// Output information in markdown format and alphabetical order
        ///
        /// Useful for creating modpack mod lists.
        /// Complements the verbose flag.
        #[arg(long, short, visible_alias = "md")]
        markdown: bool,
    },

    #[command(flatten)]
    Mgmt(MgmtCommands),
}

#[derive(Subcommand)]
pub enum MgmtCommands {
    /// Download and install everything configured in the active profile
    #[command(visible_aliases = ["install"])]
    Apply,

    /// Mark outdated mods in the active profile to be updated by the next call
    /// to `apply`
    ///
    /// Only applies to the mods/modpack added directly to the profile. Will not
    /// update individual mods inside a modpack unless they are also added to
    /// the profile
    #[command(visible_aliases = ["up"])]
    Update {
        /// Revert mods marked for updating to their installed version.
        ///
        /// Only works if updates haven't yet been applied
        #[arg(short, long)]
        revert: bool,
    },
}

#[derive(Subcommand)]
pub enum ProfileSubCommand {
    /// Show information about the current profile
    Info,
    /// List all the profiles with their data
    List,
    /// Create a new profile. Prompts when no options given
    #[command(visible_aliases = ["create"])]
    New {
        /// The Minecraft version to use
        #[arg(long, short = 'v')]
        game_version: Option<String>,
        /// The mod loader to use
        #[arg(long, short)]
        #[arg(value_enum)]
        loader: Option<ModLoader>,
        /// The name of the profile
        #[arg(long, short)]
        name: Option<String>,
        /// The directory to output mods to
        #[arg(long, short)]
        #[arg(value_hint(ValueHint::DirPath))]
        path: Option<PathBuf>,
    },
    /// Add/import and existing profile path to the config
    #[command(visible_aliases = ["import"])]
    Add {
        /// The name of the profile
        #[arg(long, short)]
        name: String,
        /// The directory containing an existing profile config file
        #[arg(value_hint(ValueHint::DirPath), default_value = DEFAULT_MINECRAFT_DIR.as_os_str())]
        path: PathBuf,
    },
    /// Delete a profile. Prompts when no options given
    #[command(visible_aliases = ["rm", "delete", "del"])]
    Remove {
        /// The name of the profile to delete
        profile_name: Option<String>,
        /// The profile to switch to afterwards
        #[arg(long, short)]
        switch_to: Option<String>,
    },
    /// Configure the current profile's name, Minecraft version, and mod loader
    #[command(visible_aliases = ["config", "conf"])]
    Configure {
        /// The Minecraft version to use
        #[arg(long, short = 'v')]
        game_version: Option<String>,
        /// The mod loader to use
        #[arg(long, short)]
        #[arg(value_enum)]
        loader: Option<ModLoader>,
        /// The name of the profile
        #[arg(long, short)]
        name: Option<String>,
    },
    /// Switch between different profiles. Prompts when no options given
    Switch {
        /// The name or path suffix of the profile to switch to
        profile_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ModpackSubCommand {
    /// Show information about the current modpack
    Info,
    /// Set a modpack on the active profile.
    #[command(visible_aliases = ["set"])]
    Add {
        /// The identifier of the modpack/project
        ///
        /// The Modrinth project ID is specified at the bottom of the left
        /// sidebar under 'Technical information'. You can also use the project
        /// slug in the URL.
        /// The CurseForge project ID is specified at the top of the right
        /// sidebar under 'About Project'.
        id: String,
        /// Whether to install the modpack's overrides to the output directory.
        /// This will overwrite existing files when installing.
        #[arg(long, short)]
        install_overrides: Option<bool>,
    },
    /// Delete modpack from the active profile.
    #[command(visible_aliases = ["rm", "delete", "del"])]
    Remove {
        /// Delete without a confirmation prompt
        #[arg(long, short)]
        force: bool,
    },
    /// Configure the current modpack's output directory and installation of
    /// overrides. Optionally, provide the settings to change as arguments.
    #[command(visible_aliases = ["config", "conf"])]
    Configure {
        /// Whether to install the modpack's overrides to the output directory.
        /// This will overwrite existing files when installing.
        #[arg(long, short)]
        install_overrides: Option<bool>,
    },
}
