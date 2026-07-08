use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Contact-bearing User-Agent. crates.io's crawler policy requires a UA that
/// identifies the requester and provides a contact channel.
pub const USER_AGENT: &str =
    "isu-rdrand-survey/0.1 (Jason Wright, ISU PhD; jason@thought.net)";

pub const DB_DUMP_URL: &str = "https://static.crates.io/db-dump.tar.gz";

/// Returns the static.crates.io download URL for a given crate version.
pub fn crate_url(name: &str, version: &str) -> String {
    format!("https://static.crates.io/crates/{name}/{name}-{version}.crate")
}

pub struct Config {
    pub scratch: PathBuf,
}

impl Config {
    pub fn resolve(override_path: Option<PathBuf>) -> Result<Self> {
        let scratch = if let Some(p) = override_path {
            p
        } else if let Ok(env) = std::env::var("CRATES_SCRATCH_DIR") {
            PathBuf::from(env)
        } else {
            let home = std::env::var("HOME").context("HOME not set")?;
            PathBuf::from(home).join("scratch").join("crates-io")
        };
        std::fs::create_dir_all(&scratch)
            .with_context(|| format!("creating scratch dir {}", scratch.display()))?;
        Ok(Self { scratch })
    }

    pub fn dbdump_dir(&self) -> PathBuf {
        self.scratch.join("db-dump")
    }

    pub fn dbdump_archive(&self) -> PathBuf {
        self.scratch.join("db-dump.tar.gz")
    }

    pub fn tarballs_dir(&self) -> PathBuf {
        self.scratch.join("tarballs")
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.scratch.join("manifest.csv")
    }

    /// Returns the on-disk path for a crate-version's .crate tarball.
    pub fn crate_path(&self, name: &str, version: &str) -> PathBuf {
        // Use the same prefix sharding that crates.io's sparse index uses to
        // keep directory sizes manageable.
        let shard = shard_for(name);
        self.tarballs_dir()
            .join(shard)
            .join(name)
            .join(format!("{name}-{version}.crate"))
    }
}

/// Mirrors the crates.io sparse-index sharding scheme.
fn shard_for(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    match bytes.len() {
        0 => "0".into(),
        1 => "1".into(),
        2 => "2".into(),
        3 => format!("3/{}", &lower[..1]),
        _ => format!("{}/{}", &lower[..2], &lower[2..4]),
    }
}

/// Convenience: ensure a parent directory exists for the given file path.
pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    Ok(())
}
