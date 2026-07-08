mod config;
mod dbdump;
mod fetch;
mod manifest;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "crawler",
    about = "Fetches the crates.io db-dump and .crate sources for the RDRAND survey"
)]
struct Cli {
    /// Scratch directory for the corpus. Defaults to $CRATES_SCRATCH_DIR or ~/scratch/crates-io.
    #[arg(long, global = true)]
    scratch: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Download and extract the crates.io db-dump.
    DbDump {
        /// Force re-download even if a dump is already present.
        #[arg(long)]
        force: bool,
    },
    /// Fetch .crate tarballs for the latest non-yanked version of each crate.
    Fetch {
        /// Cap the number of crates fetched (for smoke testing).
        #[arg(long)]
        limit: Option<usize>,
        /// Parallel fetch workers.
        #[arg(long, default_value_t = 6)]
        concurrency: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let cfg = config::Config::resolve(cli.scratch)?;
    tracing::info!(scratch = %cfg.scratch.display(), "crawler starting");

    match cli.cmd {
        Cmd::DbDump { force } => dbdump::run(&cfg, force).await,
        Cmd::Fetch { limit, concurrency } => fetch::run(&cfg, limit, concurrency).await,
    }
}
