//! CLI command implementations.

pub mod build;
pub mod config;
pub mod deploy;
pub mod init;
pub mod replay;
pub mod rollback;
pub mod versions;

use clap::{Args, Subcommand};

/// Arguments for the init command.
#[derive(Args)]
pub struct InitArgs {
    /// Workload name.
    #[arg(default_value = ".")]
    pub name: String,

    /// Template to use.
    #[arg(short, long, default_value = "basic")]
    pub template: String,

    /// Skip git initialization.
    #[arg(long)]
    pub no_git: bool,
}

/// Arguments for the build command.
#[derive(Args)]
pub struct BuildArgs {
    /// Build in release mode.
    #[arg(short, long)]
    pub release: bool,

    /// Additional features to enable.
    #[arg(short, long)]
    pub features: Vec<String>,

    /// Skip WASM optimization.
    #[arg(long)]
    pub no_optimize: bool,

    /// Path to the workload (default: current directory).
    #[arg(short, long)]
    pub path: Option<String>,
}

/// Arguments for the deploy command.
#[derive(Args)]
pub struct DeployArgs {
    /// Environment to deploy to.
    #[arg(short, long, default_value = "production")]
    pub env: String,

    /// Skip build step.
    #[arg(long)]
    pub no_build: bool,

    /// Deploy as canary.
    #[arg(long)]
    pub canary: bool,

    /// Canary percentage (0-100).
    #[arg(long)]
    pub canary_percentage: Option<u8>,

    /// Version tag.
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Skip confirmation prompt.
    #[arg(short, long)]
    pub yes: bool,

    /// Dry run (don't actually deploy).
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the versions command.
#[derive(Args)]
pub struct VersionsArgs {
    #[command(subcommand)]
    pub command: Option<VersionsCommand>,

    /// Environment to list versions for.
    #[arg(short, long, default_value = "production")]
    pub env: String,

    /// Show only the last N versions.
    #[arg(short, long)]
    pub limit: Option<usize>,
}

#[derive(Subcommand)]
pub enum VersionsCommand {
    /// List all versions.
    List,
    /// Show details for a specific version.
    Show {
        /// Version tag or ID.
        version: String,
    },
    /// Promote a version to production.
    Promote {
        /// Version tag or ID.
        version: String,
    },
    /// Delete a version.
    Delete {
        /// Version tag or ID.
        version: String,
        /// Skip confirmation.
        #[arg(short, long)]
        yes: bool,
    },
}

/// Arguments for the rollback command.
#[derive(Args)]
pub struct RollbackArgs {
    /// Version to rollback to.
    #[arg(long)]
    pub to: Option<String>,

    /// Environment.
    #[arg(short, long, default_value = "production")]
    pub env: String,

    /// Skip confirmation prompt.
    #[arg(short, long)]
    pub yes: bool,

    /// Dry run.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the replay command.
#[derive(Args)]
pub struct ReplayArgs {
    #[command(subcommand)]
    pub command: ReplayCommand,
}

#[derive(Subcommand)]
pub enum ReplayCommand {
    /// Start recording requests.
    Record {
        /// Recording name.
        #[arg(short, long)]
        name: Option<String>,

        /// Maximum requests to record.
        #[arg(long, default_value = "100")]
        max_requests: usize,

        /// Recording duration in seconds.
        #[arg(long)]
        duration: Option<u64>,
    },
    /// Stop recording.
    Stop,
    /// List recordings.
    List,
    /// Play back a recording.
    Play {
        /// Recording name or ID.
        recording: String,

        /// Target URL to replay to.
        #[arg(short, long)]
        target: Option<String>,

        /// Compare responses.
        #[arg(long)]
        diff: bool,

        /// Concurrency level.
        #[arg(short, long, default_value = "1")]
        concurrency: usize,
    },
    /// Delete a recording.
    Delete {
        /// Recording name or ID.
        recording: String,
    },
    /// Export a recording.
    Export {
        /// Recording name or ID.
        recording: String,

        /// Output file path.
        #[arg(short, long)]
        output: String,
    },
}

/// Arguments for the config command.
#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show current configuration.
    Show,
    /// Get a config value.
    Get {
        /// Config key (dot-separated).
        key: String,
    },
    /// Set a config value.
    Set {
        /// Config key (dot-separated).
        key: String,
        /// Value to set.
        value: String,
    },
    /// Initialize a new config file.
    Init {
        /// Force overwrite existing config.
        #[arg(short, long)]
        force: bool,
    },
    /// Validate the config file.
    Validate,
}
