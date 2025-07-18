#![deny(missing_docs)]

use std::path::PathBuf;

use clap::{Parser, ValueHint};
use clap_complete::Shell;
use ferrallay::{DEFAULT_MINECRAFT_DIR, config::ModLoader};

#[derive(Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help = true)]
pub struct Allaerris {
    #[command(subcommand)]
    pub subcommand: Subcommand,
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
    /// You can also use the environment variable `ALLAERRIS_CONFIG_FILE`.
    #[arg(long, short, visible_aliases = ["config", "conf"])]
    #[arg(value_hint(ValueHint::FilePath))]
    pub config_file: Option<PathBuf>,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    #[command(flatten)]
    Mods(ModsSubcommand),

    /// Add, configure, or delete the current modpack
    Modpack {
        #[command(subcommand)]
        subcommand: Option<ModpackSubcommand>,
    },

    /// Create, configure, delete, switch, or list profiles
    Profile {
        #[command(subcommand)]
        subcommand: Option<ProfileSubcommand>,
    },

    /// List all the profiles with their data
    Profiles,

    /// Simple cache management and info commands
    Cache {
        #[command(subcommand)]
        subcommand: Option<CacheSubcommand>,
    },

    /// Print shell auto completions for the specified shell
    Complete {
        /// The shell to generate auto completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(clap::Subcommand)]
pub enum ModsSubcommand {
    /// Add mods to the active profile
    Add {
        /// The id(s) of the mod/project/repository
        ///
        /// The Modrinth project ID is specified at the bottom of the left
        /// sidebar under 'Technical information'. You can also use the project
        /// slug in the URL. The CurseForge project ID is specified at
        /// the top of the right sidebar under 'About Project'.
        /// The GitHub identifier is the repository's full name, e.g.
        /// `mgziminsky/allaerris`.
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

    /// Lock the specified mods to their installed version
    Lock {
        /// IDs of mods to lock
        #[arg(required_unless_present = "all")]
        ids: Vec<String>,

        /// Treat IDs as versions instead of mods
        ///
        /// If present in the profile, the mod associated with each version will
        /// be locked to that version
        #[arg[long, short]]
        versions: bool,

        /// Also update any already locked mods to the new version
        #[arg[long, short]]
        force: bool,

        /// Lock modpack and all mods to installed version
        #[arg[long, short, conflicts_with_all = ["ids", "versions"]]]
        all: bool,
    },

    /// Remove the locked version from the specified mods
    Unlock {
        /// IDs of mods to unlock
        #[arg(required_unless_present = "all")]
        ids: Vec<String>,

        /// Unlock modpack and all mods
        #[arg[long, short, conflicts_with = "ids"]]
        all: bool,
    },

    #[command(flatten)]
    Mgmt(MgmtCommand),
}

#[derive(clap::Subcommand)]
pub enum MgmtCommand {
    /// Download and install everything configured in the active profile
    ///
    /// Any pending updates will be finalized and no longer revertable.
    #[command(visible_aliases = ["install"])]
    Apply {
        /// Always download and reinstall files without checking if they are
        /// already present
        #[arg(long, short)]
        force: bool,

        /// Don't use cache and install files directly to profile
        #[arg(long)]
        no_cache: bool,
    },

    /// Mark outdated mods in the active profile to be updated by the next call
    /// to apply
    ///
    /// Only applies to the mods/modpack added directly to the profile. Will not
    /// update individual mods inside a modpack unless they are also added to
    /// the profile
    #[command(visible_aliases = ["up"])]
    Update {
        /// Only check mods with the specified id(s)
        ids: Vec<String>,

        /// Revert mods marked for updating to their installed version.
        ///
        /// Only works if updates haven't yet been applied
        #[arg(long, short)]
        revert: bool,

        /// Immediatly apply any updates. Revert will not be possible
        #[arg(long, short, conflicts_with = "revert")]
        apply: bool,
    },

    /// Attempt to lookup all unknown files non-recursively in the profile
    /// folders and prompt adding them to the profile
    Scan {
        /// Check all files, even if they are already known to be in the profile
        #[arg(long, short)]
        all: bool,

        /// Lock the version of added mods to the installed version
        #[arg(long, short)]
        lock: bool,
    },

    /// Server management commands
    #[command(subcommand)]
    Server(ServerSubcommand),
}

#[derive(clap::Subcommand)]
pub enum ProfileSubcommand {
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
        #[arg(value_enum, long, short)]
        loader: Option<ModLoader>,
        /// The name of the profile
        #[arg(long, short)]
        name: Option<String>,
        /// The minecraft instance directory to install profile to
        #[arg(long, short)]
        #[arg(value_hint(ValueHint::DirPath))]
        path: Option<PathBuf>,
        // Profile is for a server. Skip installing client-only mods
        #[arg(long, short)]
        server: bool,
    },
    /// Add/import an existing profile path to the config
    #[command(visible_aliases = ["add"])]
    Import {
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
    #[command(visible_aliases = ["configure", "config", "conf"])]
    Edit {
        /// The Minecraft version to use
        #[arg(long, short = 'v')]
        game_version: Option<String>,
        /// The mod loader to use
        #[arg(value_enum, long, short)]
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

#[derive(clap::Subcommand)]
pub enum ModpackSubcommand {
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
    /// Configure the profile modpack's installation of overrides
    #[command(visible_aliases = ["config", "conf"])]
    Configure {
        /// Whether to install the modpack's overrides to the output directory.
        /// This may overwrite existing files when installing.
        #[arg(long, short)]
        install_overrides: Option<bool>,
    },
}

#[derive(clap::Subcommand, Default, Clone, Copy)]
pub enum CacheSubcommand {
    /// Show count and total size of cached mod files
    #[default]
    Info,

    /// Delete all files in the cache
    Clear,
}

#[derive(clap::Subcommand, Clone)]
pub enum ServerSubcommand {
    /// Install a minecraft server
    ///
    /// By default, any unspecified options will use profile values
    Install {
        /// Directory to install the server to [default: active profile]
        ///
        /// If location contains a profile, it will be used to set any
        /// unspecified options
        out: Option<PathBuf>,

        /// The modloader to install a server for, or vanilla if not specified
        #[arg(long, short)]
        loader: Option<ModLoader>,

        /// Install the latest server for this MC version
        #[arg(long, short, conflicts_with = "version")]
        minecraft: Option<String>,

        /// The exact server version to install
        ///
        /// Version format by loader:
        /// - Fabric|Quilt: MC+LOADER -> 1.21.3+0.16.9
        /// - NeoForge: MC_MINOR.MC_PATCH.LOADER -> 21.1.73 <https://docs.neoforged.net/docs/gettingstarted/versioning/#neoforge>
        /// - Forge: MC-LOADER -> 1.21.3-53.0.7
        #[arg(long, short, verbatim_doc_comment)]
        version: Option<String>,

        /// Don't use cache and download all files directly to output directory
        #[arg(long)]
        no_cache: bool,
    },
}
