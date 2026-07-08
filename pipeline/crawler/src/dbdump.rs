use crate::config::{Config, DB_DUMP_URL, USER_AGENT};
use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub async fn run(cfg: &Config, force: bool) -> Result<()> {
    let archive = cfg.dbdump_archive();
    let extract_dir = cfg.dbdump_dir();

    if archive.exists() && !force {
        tracing::info!(path = %archive.display(), "db-dump archive already present (use --force to refetch)");
    } else {
        download(&archive).await?;
    }

    let sha = sha256_file(&archive)?;
    tracing::info!(sha256 = %sha, "archive sha256");

    std::fs::create_dir_all(&extract_dir).ok();
    extract(&archive, &extract_dir)?;
    tracing::info!(extracted = %extract_dir.display(), "db-dump extracted");

    // Record the snapshot pin for reproducibility.
    let pin = format!(
        "# Auto-written by `crawler db-dump`.\nurl = \"{DB_DUMP_URL}\"\nsha256 = \"{sha}\"\nfetched_at = \"{}\"\n",
        time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)?
    );
    let pin_path = cfg.scratch.join("snapshot.toml");
    std::fs::write(&pin_path, pin)
        .with_context(|| format!("writing {}", pin_path.display()))?;
    tracing::info!(pin = %pin_path.display(), "snapshot pin written");

    Ok(())
}

async fn download(dest: &std::path::Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(60 * 30))
        .build()?;

    let resp = client.get(DB_DUMP_URL).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} ({bytes_per_sec})",
        )?
        .progress_chars("=>-"),
    );

    crate::config::ensure_parent(dest)?;
    let tmp = dest.with_extension("tar.gz.partial");
    let mut out = tokio::fs::File::create(&tmp).await?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        pb.inc(chunk.len() as u64);
        out.write_all(&chunk).await?;
    }
    out.flush().await?;
    drop(out);
    tokio::fs::rename(&tmp, dest).await?;
    pb.finish_and_clear();
    tracing::info!(path = %dest.display(), "downloaded db-dump");
    Ok(())
}

fn sha256_file(path: &std::path::Path) -> Result<String> {
    use std::io::Read;
    let mut f = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn extract(archive: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    let f = File::open(archive)?;
    let gz = GzDecoder::new(f);
    let mut tar = tar::Archive::new(gz);
    tar.set_overwrite(true);
    tar.set_preserve_permissions(false);

    // The db-dump tarball wraps everything in a top-level dated directory
    // (e.g. `2026-10-15-020000/`). We extract under `dest/` as-is and let
    // callers discover the dated subdir.
    tar.unpack(dest)
        .with_context(|| format!("extracting {} -> {}", archive.display(), dest.display()))?;

    // Sanity-check that expected CSVs landed somewhere under dest.
    let mut found_versions = false;
    for entry in walkdir(dest)? {
        if entry.file_name().and_then(|s| s.to_str()) == Some("versions.csv") {
            found_versions = true;
            break;
        }
    }
    if !found_versions {
        bail!(
            "extraction completed but versions.csv was not found under {}",
            dest.display()
        );
    }
    Ok(())
}

/// Tiny recursive directory walker so we avoid pulling in walkdir as a dep.
fn walkdir(root: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        let rd = match std::fs::read_dir(&p) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in rd.flatten() {
            let path = entry.path();
            let ty = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if ty.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    Ok(out)
}

/// Locate the dated dump directory under `dbdump_dir()`.
pub fn dump_root(cfg: &Config) -> Result<std::path::PathBuf> {
    let root = cfg.dbdump_dir();
    let mut latest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("reading {}", root.display()))?
        .flatten()
    {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        // The dump always contains a `versions.csv` either at top level or
        // under `data/`. Treat any subdirectory that has one as a candidate.
        let candidate = entry.path();
        if has_versions_csv(&candidate) {
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            if latest.as_ref().is_none_or(|(_, t)| mtime >= *t) {
                latest = Some((candidate, mtime));
            }
        }
    }
    latest
        .map(|(p, _)| p)
        .ok_or_else(|| anyhow::anyhow!("no dump directory with versions.csv under {}", root.display()))
}

fn has_versions_csv(dir: &std::path::Path) -> bool {
    if dir.join("versions.csv").is_file() {
        return true;
    }
    if dir.join("data").join("versions.csv").is_file() {
        return true;
    }
    false
}

/// Resolve the path of a named CSV inside the dump (handles both
/// flat-at-root and `data/`-nested layouts).
pub fn dump_csv(root: &std::path::Path, name: &str) -> Result<std::path::PathBuf> {
    let a = root.join(name);
    if a.is_file() {
        return Ok(a);
    }
    let b = root.join("data").join(name);
    if b.is_file() {
        return Ok(b);
    }
    bail!("could not find {name} under {}", root.display())
}

// Silence the unused warning when only the binary `db-dump` subcommand runs.
#[allow(dead_code)]
fn _ensure_write_trait(_: &mut dyn Write) {}
