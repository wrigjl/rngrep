//! Detect SGX-targeted crates from `Cargo.toml` signals.
//!
//! Returns the matched signals if any apply; `None` otherwise. See
//! `outofscope.rs` for how this result is folded into the overall
//! out-of-scope classification.

/// Crate-name pattern matches that strongly imply an SGX target.
fn name_signal(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    if n == "sallyport" || n == "enarx" {
        return Some("name");
    }
    if n.starts_with("enarx-") || n.starts_with("enarx_") {
        return Some("name:enarx-prefix");
    }
    if n.ends_with("-sgx") || n.ends_with("_sgx") {
        return Some("name:sgx-suffix");
    }
    if n.starts_with("sgx-") || n.starts_with("sgx_") {
        return Some("name:sgx-prefix");
    }
    None
}

/// Direct dependency names that imply an SGX target.
fn dep_signal(dep_name: &str) -> Option<&'static str> {
    let d = dep_name.to_ascii_lowercase();
    if matches!(
        d.as_str(),
        "sgx"
            | "sgx_tstd"
            | "sgx_types"
            | "sgx_urts"
            | "sgx_tcrypto"
            | "sgx_alloc"
            | "sgx_libc"
            | "sgx-isa"
            | "sallyport"
            | "aesm-client"
            | "enclave-runner"
    ) {
        return Some("dep");
    }
    if d.starts_with("fortanix-sgx") || d.starts_with("fortanix_sgx") {
        return Some("dep:fortanix-sgx");
    }
    if d.starts_with("sgx-isa") || d.starts_with("sgx_isa") {
        return Some("dep:sgx-isa");
    }
    None
}

/// Returns SGX signals if the crate appears SGX-targeted, else `None`.
pub fn detect(crate_name: &str, cargo_tomls: &[(&str, &str)]) -> Option<Vec<String>> {
    let mut signals: Vec<String> = Vec::new();
    if let Some(s) = name_signal(crate_name) {
        signals.push(s.into());
    }
    for (_path, src) in cargo_tomls {
        let Ok(val) = toml::from_str::<toml::Value>(src) else {
            continue;
        };
        if val
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("fortanix-sgx"))
            .is_some()
        {
            signals.push("metadata:fortanix-sgx".into());
        }
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(deps) = val.get(section).and_then(|v| v.as_table()) {
                for (dep, _) in deps {
                    if let Some(s) = dep_signal(dep) {
                        signals.push(format!("{section}:{dep}:{s}"));
                    }
                }
            }
        }
        if let Some(target) = val.get("target").and_then(|v| v.as_table()) {
            for (_t, sub) in target {
                for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some(deps) = sub.get(section).and_then(|v| v.as_table()) {
                        for (dep, _) in deps {
                            if let Some(s) = dep_signal(dep) {
                                signals.push(format!("target.{section}:{dep}:{s}"));
                            }
                        }
                    }
                }
            }
        }
        if let Some(t) = val
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("docs"))
            .and_then(|d| d.get("rs"))
            .and_then(|r| r.get("default-target"))
            .and_then(|v| v.as_str())
            && (t.contains("sgx") || t.contains("fortanix"))
        {
            signals.push(format!("metadata:default-target:{t}"));
        }
    }
    let mut seen = std::collections::HashSet::new();
    signals.retain(|s| seen.insert(s.clone()));
    if signals.is_empty() {
        None
    } else {
        Some(signals)
    }
}
