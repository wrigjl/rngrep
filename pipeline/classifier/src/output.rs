use crate::classify::{CallsiteRow, SummaryRow};
use anyhow::Result;
use std::fs::OpenOptions;
use std::path::Path;

pub struct CallsitesWriter {
    inner: csv::Writer<std::fs::File>,
}

impl CallsitesWriter {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).ok();
        }
        // Always rewrite on a fresh run; the classifier is fast enough that
        // resume isn't useful and the callsites file is bounded.
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        let mut w = csv::WriterBuilder::new().has_headers(false).from_writer(f);
        w.write_record([
            "name",
            "version",
            "file",
            "line",
            "kind",
            "detail",
            "cfg_gates",
            "in_test_or_bench",
            "runtime_gated",
            "compile_gate_present",
            "target_feature_attr",
            "is_out_of_scope",
            "out_of_scope_reason",
            "category",
        ])?;
        w.flush()?;
        Ok(Self { inner: w })
    }

    pub fn write(&mut self, row: &CallsiteRow) -> Result<()> {
        self.inner.serialize(row)?;
        Ok(())
    }
}

pub struct CratesWriter {
    inner: csv::Writer<std::fs::File>,
}

impl CratesWriter {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).ok();
        }
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        let mut w = csv::WriterBuilder::new().has_headers(false).from_writer(f);
        w.write_record([
            "name",
            "version",
            "num_callsites",
            "num_intrinsic",
            "num_inline_asm",
            "n_runtime",
            "n_compile_gate",
            "n_unconditional",
            "n_out_of_scope",
            "n_in_test",
            "category",
            "build_config_exposure",
            "build_config_detail",
            "is_out_of_scope",
            "out_of_scope_reason",
            "out_of_scope_signals",
            "parse_failures",
        ])?;
        w.flush()?;
        Ok(Self { inner: w })
    }

    pub fn write(&mut self, row: &SummaryRow) -> Result<()> {
        // Skip rows with no findings of any kind to keep the summary file
        // focused. Crates with build-config exposure are kept regardless.
        if row.num_callsites == 0 && !row.build_config_exposure {
            return Ok(());
        }
        self.inner.serialize(row)?;
        Ok(())
    }
}
