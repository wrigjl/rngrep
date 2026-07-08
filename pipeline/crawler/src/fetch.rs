use crate::config::{Config, USER_AGENT, crate_url, ensure_parent};
use crate::dbdump;
use crate::manifest::{ManifestRow, Writer};
use anyhow::{Context, Result, bail};
use futures::stream::{FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};

/// Subset of crates.csv columns we care about.
#[derive(Debug, Deserialize)]
struct CrateRow {
    id: u64,
    name: String,
}

/// Subset of versions.csv columns we care about.
#[derive(Debug, Deserialize)]
struct VersionRow {
    crate_id: u64,
    num: String,
    created_at: String,
    yanked: String,
    checksum: String,
}

#[derive(Debug, Clone)]
struct Target {
    name: String,
    version: String,
    /// Expected sha256 from versions.csv (hex).
    checksum: String,
}

pub async fn run(cfg: &Config, limit: Option<usize>, concurrency: usize) -> Result<()> {
    let dump_root = dbdump::dump_root(cfg)
        .context("locate dump root — run `crawler db-dump` first")?;
    tracing::info!(dump = %dump_root.display(), "using db-dump");

    let crates_csv = dbdump::dump_csv(&dump_root, "crates.csv")?;
    let versions_csv = dbdump::dump_csv(&dump_root, "versions.csv")?;

    let crate_names = load_crate_names(&crates_csv)?;
    tracing::info!(crates = crate_names.len(), "loaded crate name map");

    let mut targets = pick_latest(&crate_names, &versions_csv)?;
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    tracing::info!(targets = targets.len(), "selected latest non-yanked versions");

    if let Some(n) = limit {
        targets.truncate(n);
        tracing::info!(limit = n, "limit applied");
    }

    let existing = crate::manifest::load(&cfg.manifest_path())?;
    let writer = Arc::new(Mutex::new(Writer::open(&cfg.manifest_path())?));
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(120))
        .build()?;

    let pb = ProgressBar::new(targets.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40}] {pos}/{len} {msg}",
        )?
        .progress_chars("=>-"),
    );

    let sem = Arc::new(Semaphore::new(concurrency.max(1)));
    let mut tasks = FuturesUnordered::new();
    for target in targets {
        let key = (target.name.clone(), target.version.clone());
        let on_disk = cfg.crate_path(&target.name, &target.version);
        if existing.contains_key(&key) && on_disk.is_file() {
            pb.inc(1);
            continue;
        }

        let permit = sem.clone().acquire_owned().await?;
        let client = client.clone();
        let writer = writer.clone();
        let pb = pb.clone();
        let out_path = on_disk;
        let cfg_scratch = cfg.scratch.clone();
        let _ = cfg_scratch;

        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            let r = fetch_one(&client, &target, &out_path).await;
            match r {
                Ok(row) => {
                    let mut w = writer.lock().await;
                    if let Err(e) = w.append(&row) {
                        tracing::warn!(name = %row.name, version = %row.version, error = %e, "manifest append failed");
                    }
                    pb.inc(1);
                    pb.set_message(format!("{}-{}", row.name, row.version));
                }
                Err(e) => {
                    pb.inc(1);
                    tracing::warn!(name = %target.name, version = %target.version, error = %e, "fetch failed");
                }
            }
        }));

        // Drain a few completions to bound the in-flight set size.
        while tasks.len() >= concurrency * 4 {
            tasks.next().await;
        }
    }

    while tasks.next().await.is_some() {}
    pb.finish_with_message("done");
    Ok(())
}

fn load_crate_names(path: &std::path::Path) -> Result<HashMap<u64, String>> {
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening {}", path.display()))?;
    let mut out = HashMap::new();
    for row in rdr.deserialize::<CrateRow>() {
        let row = row?;
        out.insert(row.id, row.name);
    }
    Ok(out)
}

fn pick_latest(
    crate_names: &HashMap<u64, String>,
    versions_csv: &std::path::Path,
) -> Result<Vec<Target>> {
    // Walk versions.csv once, keeping the most-recently-created non-yanked
    // non-prerelease version per crate. Prerelease is identified by a `-` in
    // the semver string.
    let mut rdr = csv::Reader::from_path(versions_csv)
        .with_context(|| format!("opening {}", versions_csv.display()))?;
    let mut best: HashMap<u64, VersionRow> = HashMap::new();
    for row in rdr.deserialize::<VersionRow>() {
        let row = row?;
        if is_yanked(&row.yanked) {
            continue;
        }
        if row.num.contains('-') {
            continue;
        }
        match best.get(&row.crate_id) {
            None => {
                best.insert(row.crate_id, row);
            }
            Some(prev) if row.created_at > prev.created_at => {
                best.insert(row.crate_id, row);
            }
            _ => {}
        }
    }
    let mut out = Vec::with_capacity(best.len());
    for (id, v) in best {
        if let Some(name) = crate_names.get(&id) {
            out.push(Target {
                name: name.clone(),
                version: v.num,
                checksum: v.checksum,
            });
        }
    }
    Ok(out)
}

/// crates.io's `yanked` column is `t`/`f`.
fn is_yanked(s: &str) -> bool {
    matches!(s, "t" | "true" | "TRUE" | "1")
}

async fn fetch_one(
    client: &reqwest::Client,
    target: &Target,
    out_path: &std::path::Path,
) -> Result<ManifestRow> {
    let url = crate_url(&target.name, &target.version);
    ensure_parent(out_path)?;

    // If the file is already on disk with a matching checksum, reuse it.
    if out_path.is_file()
        && let Ok((sha, size)) = sha_and_size(out_path)
        && sha == target.checksum
    {
        return Ok(ManifestRow {
            name: target.name.clone(),
            version: target.version.clone(),
            sha256: sha,
            size_bytes: size,
            fetched_at: now_rfc3339()?,
        });
    }

    let resp = client.get(&url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha = hex::encode(hasher.finalize());
    if sha != target.checksum {
        bail!(
            "checksum mismatch for {}-{}: expected {}, got {}",
            target.name,
            target.version,
            target.checksum,
            sha
        );
    }

    let tmp = out_path.with_extension("crate.partial");
    tokio::fs::write(&tmp, &bytes).await?;
    tokio::fs::rename(&tmp, out_path).await?;

    Ok(ManifestRow {
        name: target.name.clone(),
        version: target.version.clone(),
        sha256: sha,
        size_bytes: bytes.len() as u64,
        fetched_at: now_rfc3339()?,
    })
}

fn sha_and_size(path: &std::path::Path) -> Result<(String, u64)> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let size = f.metadata()?.len();
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok((hex::encode(hasher.finalize()), size))
}

fn now_rfc3339() -> Result<String> {
    Ok(time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)?)
}
