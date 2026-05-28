// SPDX-License-Identifier: AGPL-3.0-or-later
//! `foundation` `UniBin` — scientific validation CLI for `projectFOUNDATION`.
//!
//! Subcommands:
//! - `validate` — run the 8-phase validation pipeline
//! - `fetch` — download data sources from manifests
//! - `health` — check primal health triad
//! - `targets` — inspect and verify target manifests
//! - `backfill` — populate BLAKE3 hashes in source manifests

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;

/// foundation — scientific validation for ecoPrimals.
#[derive(Parser)]
#[command(name = "foundation", version, about, long_about = None)]
struct Cli {
    /// Project root directory (defaults to current directory).
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,

    /// Logging verbosity (repeat for more: -v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the 8-phase validation pipeline.
    Validate {
        /// Restrict to a specific thread (short name or ID).
        #[arg(long)]
        thread: Option<String>,
        /// Skip the data fetch phase.
        #[arg(long)]
        skip_fetch: bool,
        /// Data directory for fetched sources.
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
    /// Fetch data sources from manifests.
    Fetch {
        /// Restrict to a specific thread.
        #[arg(long)]
        thread: Option<String>,
        /// Data directory for output.
        #[arg(long)]
        data_dir: Option<PathBuf>,
        /// Also register artifacts in `NestGate`.
        #[arg(long)]
        register: bool,
    },
    /// Check health triad of NUCLEUS primals.
    Health {
        /// Show detailed per-primal status.
        #[arg(long)]
        verbose: bool,
    },
    /// Inspect and verify target manifests.
    Targets {
        /// Restrict to a specific thread.
        #[arg(long)]
        thread: Option<String>,
        /// Verify target counts and schemas.
        #[arg(long)]
        check: bool,
    },
    /// Populate BLAKE3 hashes in source manifests (backfill).
    Backfill {
        /// Data directory containing fetched files.
        #[arg(long)]
        data_dir: Option<PathBuf>,
        /// Show what would change without modifying files.
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let filter = match cli.verbose {
        0 => "foundation=info",
        1 => "foundation=debug",
        _ => "foundation=trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .compact()
        .init();

    let result = match cli.command {
        Commands::Validate {
            thread,
            skip_fetch,
            data_dir,
        } => commands::validate(cli.root, thread, skip_fetch, data_dir).await,
        Commands::Fetch {
            thread,
            data_dir,
            register,
        } => commands::fetch(cli.root, thread, data_dir, register).await,
        Commands::Health { verbose } => commands::health(cli.root, verbose).await,
        Commands::Targets { thread, check } => commands::targets(cli.root, thread, check).await,
        Commands::Backfill { data_dir, dry_run } => {
            commands::backfill(cli.root, data_dir, dry_run).await
        }
    };

    if let Err(e) = result {
        tracing::error!(error = %e, "command failed");
        std::process::exit(1);
    }
}
