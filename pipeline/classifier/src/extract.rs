use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum EntryKind {
    Rust,
    CargoToml,
    CargoConfig,
    BuildRs,
}

pub struct Entry {
    pub path: String,
    pub kind: EntryKind,
    pub contents: Vec<u8>,
}

/// Iterate the source files we care about out of a `.crate` tarball.
///
/// A `.crate` is a gzipped tarball whose first-level directory is
/// `<name>-<version>/`. We strip that prefix when emitting the entry path
/// so downstream code sees stable relative paths.
pub fn read_crate(path: &Path) -> Result<Vec<Entry>> {
    let f = std::fs::File::open(path)
        .with_context(|| format!("opening {}", path.display()))?;
    let gz = GzDecoder::new(f);
    let mut tar = tar::Archive::new(gz);
    let mut out = Vec::new();

    for entry in tar.entries()? {
        let mut entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path_in_tar = match entry.path() {
            Ok(p) => p.into_owned(),
            Err(_) => continue,
        };
        let mut comps = path_in_tar.components();
        // skip the leading `<name>-<version>/` directory
        comps.next();
        let rel: std::path::PathBuf = comps.collect();
        if rel.as_os_str().is_empty() {
            continue;
        }

        let kind = classify_path(&rel);
        let Some(kind) = kind else { continue };

        // 8 MB cap per file to bound pathological inputs.
        const MAX_BYTES: usize = 8 * 1024 * 1024;
        let mut buf = Vec::new();
        let size = entry.header().size().unwrap_or(0);
        if size > MAX_BYTES as u64 {
            continue;
        }
        entry.read_to_end(&mut buf)?;

        out.push(Entry {
            path: rel.to_string_lossy().into_owned(),
            kind,
            contents: buf,
        });
    }
    Ok(out)
}

fn classify_path(rel: &Path) -> Option<EntryKind> {
    let s = rel.to_string_lossy();
    let lower = s.to_ascii_lowercase();
    if lower.ends_with("/cargo.toml") || lower == "cargo.toml" {
        return Some(EntryKind::CargoToml);
    }
    if lower.ends_with(".cargo/config.toml")
        || lower.ends_with(".cargo/config")
        || lower == ".cargo/config.toml"
        || lower == ".cargo/config"
    {
        return Some(EntryKind::CargoConfig);
    }
    if lower.ends_with("/build.rs") || lower == "build.rs" {
        return Some(EntryKind::BuildRs);
    }
    if lower.ends_with(".rs") {
        return Some(EntryKind::Rust);
    }
    None
}
