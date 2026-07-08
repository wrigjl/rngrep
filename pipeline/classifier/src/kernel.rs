//! Detect crates that run as a kernel, guest kernel, or unikernel —
//! code that is not a Linux userspace process and therefore not
//! recordable by rr.
//!
//! Known targets at detection time:
//!   - HermitOS family (`libhermit-rs`, `rusty-hermit`, and crates
//!     depending on the Hermit runtime).
//!   - Asterinas (`ostd` and crates depending on Asterinas/ostd).
//!
//! Detection signals (any one suffices):
//!   - Crate name in the Hermit / Asterinas family.
//!   - Dep on a known kernel-side crate (`hermit-entry`,
//!     `hermit-sync`, `hermit-abi`, `ostd-*`, `asterinas-*`,
//!     `tdx-guest`).
//!   - Keywords include `unikernel` or `libos`.
//!   - Description mentions "unikernel" or "library operating
//!     system".

fn name_signal(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();
    if n == "ostd" {
        return Some("name");
    }
    if n.starts_with("libhermit-") || n.starts_with("libhermit_") || n == "libhermit-rs" {
        return Some("name:libhermit");
    }
    if n.starts_with("rusty-hermit") || n.starts_with("rusty_hermit") {
        return Some("name:rusty-hermit");
    }
    if n.starts_with("hermit-") || n.starts_with("hermit_") {
        return Some("name:hermit-prefix");
    }
    if n.ends_with("-hermit") || n.ends_with("_hermit") {
        return Some("name:hermit-suffix");
    }
    if n.starts_with("ostd-") || n.starts_with("ostd_") || n.starts_with("asterinas-") || n.starts_with("asterinas_") {
        return Some("name:asterinas-family");
    }
    None
}

fn dep_signal(dep: &str) -> Option<&'static str> {
    let d = dep.to_ascii_lowercase();
    if matches!(
        d.as_str(),
        "hermit-entry"
            | "hermit-sync"
            | "hermit-abi"
            | "hermit-macro"
            | "tdx-guest"
    ) {
        return Some("dep");
    }
    if d.starts_with("ostd-") || d.starts_with("ostd_") {
        return Some("dep:ostd");
    }
    if d.starts_with("asterinas-") || d.starts_with("asterinas_") {
        return Some("dep:asterinas");
    }
    None
}

fn description_signal(desc: &str) -> Option<&'static str> {
    let d = desc.to_ascii_lowercase();
    if d.contains("unikernel") {
        return Some("description:unikernel");
    }
    if d.contains("library operating system") || d.contains("library-based operating system") {
        return Some("description:library-os");
    }
    None
}

fn keyword_signal(kw: &str) -> Option<&'static str> {
    let k = kw.to_ascii_lowercase();
    if k == "unikernel" {
        return Some("keyword:unikernel");
    }
    if k == "libos" {
        return Some("keyword:libos");
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
    }
    let mut seen = std::collections::HashSet::new();
    signals.retain(|s| seen.insert(s.clone()));
    if signals.is_empty() {
        None
    } else {
        Some(signals)
    }
}
