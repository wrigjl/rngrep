use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRow {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub fetched_at: String,
}

/// Load an existing manifest into a (name, version) -> row map.
pub fn load(path: &Path) -> Result<HashMap<(String, String), ManifestRow>> {
    let mut out = HashMap::new();
    if !path.exists() {
        return Ok(out);
    }
    let mut rdr = csv::Reader::from_path(path)
        .with_context(|| format!("opening manifest {}", path.display()))?;
    for row in rdr.deserialize::<ManifestRow>() {
        let row = row?;
        out.insert((row.name.clone(), row.version.clone()), row);
    }
    Ok(out)
}

/// Append-only writer. Opens (or creates) the manifest and writes a header
/// on first creation.
pub struct Writer {
    inner: csv::Writer<std::fs::File>,
}

impl Writer {
    pub fn open(path: &Path) -> Result<Self> {
        let needs_header = !path.exists();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("opening manifest {}", path.display()))?;
        let mut w = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(f);
        if needs_header {
            w.write_record(["name", "version", "sha256", "size_bytes", "fetched_at"])?;
            w.flush()?;
        }
        Ok(Self { inner: w })
    }

    pub fn append(&mut self, row: &ManifestRow) -> Result<()> {
        self.inner.serialize(row)?;
        self.inner.flush()?;
        Ok(())
    }
}
