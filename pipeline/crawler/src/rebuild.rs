//! Regenerates manifest.csv from a corpus that's already on disk (e.g. one
//! extracted from an archival corpus.tar), without needing to re-fetch or
//! consult crates.io. Everything a manifest row needs except `fetched_at`
//! is recoverable directly from the tarball tree itself: `name` is the
//! parent directory (unambiguous even for hyphenated crate names, unlike
//! parsing it back out of `<name>-<version>.crate`), `version` is what's
//! left of the filename after stripping that name and `.crate`, and
//! `sha256`/`size_bytes` are computed directly from the file's actual
//! bytes -- which also means a rebuilt manifest is self-verifying rather
//! than trusting a separately-shipped file. `fetched_at` is dead weight
//! downstream (classifier's ManifestRow carries it but never reads it,
//! and revdep doesn't consume manifest.csv at all), so it's left blank.

use crate::config::Config;
use crate::manifest::{ManifestRow, Writer};
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn run(cfg: &Config) -> Result<()> {
    let tarballs_dir = cfg.tarballs_dir();
    if !tarballs_dir.is_dir() {
        bail!("tarballs dir not found: {}", tarballs_dir.display());
    }
    let manifest_path = cfg.manifest_path();
    if manifest_path.exists() {
        bail!(
            "{} already exists -- remove it first if you intend to rebuild \
             (this tool only appends to an empty manifest, to avoid silently \
             mixing rebuilt rows with whatever was already there)",
            manifest_path.display()
        );
    }

    let mut files = Vec::new();
    walk(&tarballs_dir, &mut files)?;
    files.sort();
    tracing::info!(found = files.len(), "found .crate files");

    let mut writer = Writer::open(&manifest_path)?;
    for (i, path) in files.iter().enumerate() {
        let row = row_for(path)?;
        writer.append(&row)?;
        if (i + 1) % 20_000 == 0 {
            tracing::info!(done = i + 1, total = files.len(), "hashing");
        }
    }
    tracing::info!(
        count = files.len(),
        manifest = %manifest_path.display(),
        "rebuilt manifest"
    );
    Ok(())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            walk(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "crate") {
            out.push(path);
        }
    }
    Ok(())
}

fn row_for(path: &Path) -> Result<ManifestRow> {
    // path = .../tarballs/<shard...>/<name>/<name>-<version>.crate
    let name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .with_context(|| format!("cannot determine crate name from {}", path.display()))?
        .to_string();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("bad filename {}", path.display()))?;
    let stem = filename
        .strip_suffix(".crate")
        .with_context(|| format!("expected .crate suffix: {filename}"))?;
    let version = stem
        .strip_prefix(&format!("{name}-"))
        .with_context(|| format!("filename {filename} does not start with {name}-"))?
        .to_string();

    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let size_bytes = bytes.len() as u64;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256 = hex::encode(hasher.finalize());

    Ok(ManifestRow {
        name,
        version,
        sha256,
        size_bytes,
        fetched_at: String::new(),
    })
}
