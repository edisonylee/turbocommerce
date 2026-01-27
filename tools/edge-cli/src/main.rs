//! Edge CLI - Command line tool for the edge streaming SSR platform.
//!
//! Commands:
//! - `edge init` - Initialize a new workload
//! - `edge build` - Build a workload for deployment
//! - `edge deploy` - Deploy a workload
//! - `edge versions` - List deployed versions
//! - `edge rollback` - Rollback to a previous version
//! - `edge replay` - Record and replay requests
//! - `edge config` - Manage configuration

mod commands;
mod config;
mod context;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{BuildArgs, ConfigArgs, DeployArgs, InitArgs, ReplayArgs, RollbackArgs, VersionsArgs};

/// Edge CLI - Deploy and manage edge streaming SSR workloads
#[derive(Parser)]
#[command(name = "edge")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Use JSON output format
    #[arg(long, global = true)]
    json: bool,

    /// Config file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new workload project
    Init(InitArgs),

    /// Build a workload for deployment
    Build(BuildArgs),

    /// Deploy a workload to the edge
    Deploy(DeployArgs),

    /// List deployed versions
    Versions(VersionsArgs),

    /// Rollback to a previous version
    Rollback(RollbackArgs),

    /// Record and replay requests for debugging
    Replay(ReplayArgs),

    /// Manage configuration
    Config(ConfigArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup output formatting
    let output = output::Output::new(cli.verbose, cli.json);

    // Load config
    let config_path = cli.config.as_deref();
    let ctx = context::Context::load(config_path, output)?;

    // Execute command
    let result = match cli.command {
        Commands::Init(args) => commands::init::run(args, &ctx).await,
        Commands::Build(args) => commands::build::run(args, &ctx).await,
        Commands::Deploy(args) => commands::deploy::run(args, &ctx).await,
        Commands::Versions(args) => commands::versions::run(args, &ctx).await,
        Commands::Rollback(args) => commands::rollback::run(args, &ctx).await,
        Commands::Replay(args) => commands::replay::run(args, &ctx).await,
        Commands::Config(args) => commands::config::run(args, &ctx).await,
    };

    if let Err(e) = result {
        ctx.output.error(&format!("{:#}", e));
        std::process::exit(1);
    }

    Ok(())
}
