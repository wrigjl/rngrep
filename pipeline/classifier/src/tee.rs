//! Detect TEE-targeted crates that are not caught by the SGX-specific
//! detector. These run inside confidential-compute environments
//! (Intel TDX, AMD SEV-SNP, Arm CCA, etc.) where CPUID is unreliable
//! and rr cannot record process state regardless.
//!
//! Detection signals:
//!   - Keywords include `tee`.
//!   - Description mentions `TEE`, `TEEs`, or `trusted execution`.
//!   - Crate name contains a `-tee-` or `-tee_` segment.

fn name_signal(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    // Match `tee` as a hyphen/underscore-delimited segment to avoid
    // false positives like `teevec`, `committee`, or `cofee`.
    let segments: Vec<&str> = n.split(['-', '_']).collect();
    if segments.iter().any(|s| *s == "tee" || *s == "tees") {
        return Some("name:tee-segment");
    }
    None
}

fn description_signal(desc: &str) -> Option<&'static str> {
    let d = desc.to_lowercase();
    // Whole-word match for "tee" / "tees" / "trusted execution".
    let has_tee = d
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|w| w == "tee" || w == "tees");
    if has_tee {
        return Some("description:tee");
    }
    if d.contains("trusted execution environment") || d.contains("trusted execution") {
        return Some("description:trusted-execution");
    }
    None
}

fn keyword_signal(kw: &str) -> Option<&'static str> {
    if kw.eq_ignore_ascii_case("tee") {
        return Some("keyword:tee");
    }
    None
}

pub fn detect(crate_name: &str, cargo_tomls: &[(&str, &str)]) -> Option<Vec<String>> {
    let mut signals: Vec<String> = Vec::new();
    if let Some(s) = name_signal(crate_name) {
        signals.push(s.into());
    }
    for (_path, src) in cargo_tomls {
        let Ok(val) = toml::from_str::<toml::Value>(src) else {
            continue;
        };
        let pkg = val.get("package");
        if let Some(desc) = pkg.and_then(|p| p.get("description")).and_then(|v| v.as_str())
            && let Some(s) = description_signal(desc)
        {
            signals.push(s.into());
        }
        if let Some(kws) = pkg.and_then(|p| p.get("keywords")).and_then(|v| v.as_array()) {
            for kw in kws {
                if let Some(s) = kw.as_str().and_then(keyword_signal) {
                    signals.push(s.into());
                }
            }
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
