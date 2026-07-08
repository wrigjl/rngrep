mod ast;
mod buildconfig;
mod classify;
mod extract;
mod kernel;
mod outofscope;
mod output;
mod prefilter;
mod sgx;
mod tee;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Parser)]
#[command(
    name = "classifier",
    about = "Static-analysis classifier for RDRAND/RDSEED usage in Rust crates"
)]
struct Cli {
    /// Path to the crawler's manifest.csv. Defaults to
    /// $CRATES_SCRATCH_DIR/manifest.csv or ~/scratch/crates-io/manifest.csv.
    #[arg(long)]
    manifest: Option<PathBuf>,

    /// Path to the scratch directory containing tarballs/. Defaults to the
    /// manifest's parent directory.
    #[arg(long)]
    scratch: Option<PathBuf>,

    /// Output directory for results CSVs.
    #[arg(long, default_value = "results")]
    out: PathBuf,

    /// Cap the number of crates classified (for smoke testing).
    #[arg(long)]
    limit: Option<usize>,

    /// Only classify these crates (comma-separated names).
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,

    /// Rayon thread pool size. 0 means default (one per logical core).
    #[arg(long, default_value_t = 0)]
    threads: usize,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct ManifestRow {
    name: String,
    version: String,
    #[allow(dead_code)]
    sha256: String,
    #[allow(dead_code)]
    size_bytes: u64,
    #[allow(dead_code)]
    fetched_at: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let cli = Cli::parse();

    let manifest_path = cli.manifest.unwrap_or_else(|| {
        if let Ok(s) = std::env::var("CRATES_SCRATCH_DIR") {
            PathBuf::from(s).join("manifest.csv")
        } else {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home)
                .join("scratch")
                .join("crates-io")
                .join("manifest.csv")
        }
    });
    let scratch = cli
        .scratch
        .clone()
        .unwrap_or_else(|| manifest_path.parent().unwrap().to_path_buf());
    std::fs::create_dir_all(&cli.out)?;

    if cli.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.threads)
            .build_global()
            .ok();
    }

    let only: std::collections::HashSet<String> = cli.only.into_iter().collect();
    let rows = load_manifest(&manifest_path, &only, cli.limit)?;
    tracing::info!(rows = rows.len(), manifest = %manifest_path.display(), "loaded manifest");

    let pb = ProgressBar::new(rows.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40}] {pos}/{len} {msg}",
        )?
        .progress_chars("=>-"),
    );

    let callsites_out = cli.out.join("callsites.csv");
    let crates_out = cli.out.join("crates.csv");
    let callsites_w = Mutex::new(output::CallsitesWriter::open(&callsites_out)?);
    let crates_w = Mutex::new(output::CratesWriter::open(&crates_out)?);

    // AArch64 RNDR/RNDRRS results, same schema, separate files.
    let arm_callsites_out = cli.out.join("arm_callsites.csv");
    let arm_crates_out = cli.out.join("arm_crates.csv");
    let arm_callsites_w = Mutex::new(output::CallsitesWriter::open(&arm_callsites_out)?);
    let arm_crates_w = Mutex::new(output::CratesWriter::open(&arm_crates_out)?);

    let tarballs_dir = scratch.join("tarballs");

    rows.par_iter().for_each(|row| {
        let path = crate_path(&tarballs_dir, &row.name, &row.version);
        let result = classify::classify_crate(&row.name, &row.version, &path);
        match result {
            Ok(rep) => {
                if let Ok(mut w) = callsites_w.lock() {
                    for cs in &rep.callsites {
                        if let Err(e) = w.write(cs) {
                            tracing::warn!(name = %row.name, version = %row.version, error = %e, "callsite write failed");
                        }
                    }
                }
                if let Ok(mut w) = crates_w.lock()
                    && let Err(e) = w.write(&rep.summary)
                {
                    tracing::warn!(name = %row.name, version = %row.version, error = %e, "summary write failed");
                }
                if let Ok(mut w) = arm_callsites_w.lock() {
                    for cs in &rep.arm_callsites {
                        if let Err(e) = w.write(cs) {
                            tracing::warn!(name = %row.name, version = %row.version, error = %e, "arm callsite write failed");
                        }
                    }
                }
                if let Ok(mut w) = arm_crates_w.lock()
                    && let Err(e) = w.write(&rep.arm_summary)
                {
                    tracing::warn!(name = %row.name, version = %row.version, error = %e, "arm summary write failed");
                }
            }
            Err(e) => {
                tracing::debug!(name = %row.name, version = %row.version, error = %e, "classify failed");
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    tracing::info!(
        callsites = %callsites_out.display(),
        crates = %crates_out.display(),
        "classification complete"
    );
    Ok(())
}

fn load_manifest(
    path: &std::path::Path,
    only: &std::collections::HashSet<String>,
    limit: Option<usize>,
) -> Result<Vec<ManifestRow>> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening manifest {}", path.display()))?;
    let mut out = Vec::new();
    for row in rdr.deserialize::<ManifestRow>() {
        let row = row?;
        if !only.is_empty() && !only.contains(&row.name) {
            continue;
        }
        out.push(row);
    }
    if let Some(n) = limit {
        out.truncate(n);
    }
    Ok(out)
}

/// Mirrors the crawler's sharding scheme.
fn crate_path(tarballs: &std::path::Path, name: &str, version: &str) -> PathBuf {
    let lower = name.to_ascii_lowercase();
    let shard = match lower.len() {
        0 => "0".to_string(),
        1 => "1".to_string(),
        2 => "2".to_string(),
        3 => format!("3/{}", &lower[..1]),
        _ => format!("{}/{}", &lower[..2], &lower[2..4]),
    };
    tarballs
        .join(shard)
        .join(name)
        .join(format!("{name}-{version}.crate"))
}
