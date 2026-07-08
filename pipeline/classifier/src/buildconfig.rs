use serde::Deserialize;

/// What a crate's own packaging exposes about build flags. We only inspect
/// files inside the .crate tarball; project-level RUSTFLAGS set on the
/// build machine are invisible from here and covered by the methodology
/// discussion.
#[derive(Debug, Default, Clone)]
pub struct BuildConfigFinding {
    /// File the finding came from (e.g. `Cargo.toml`, `.cargo/config.toml`).
    pub file: String,
    /// Raw rustflags value detected.
    pub detail: String,
}

impl BuildConfigFinding {
    /// Whether this flag exposes the x86 hardware-entropy instructions.
    /// `+rdrand`/`+rdseed` are explicit; `target-cpu=native` is ISA-neutral
    /// (it enables whatever the build CPU has, so it exposes x86 on an x86
    /// host).
    pub fn exposes_x86(&self) -> bool {
        let l = self.detail.to_ascii_lowercase();
        l.contains("+rdrand")
            || l.contains("+rdseed")
            || l.contains("target-cpu=native")
    }

    /// Whether this flag exposes the AArch64 hardware-entropy instructions.
    /// `target-feature=+rand` (FEAT_RNG) is explicit; `target-cpu=native`
    /// exposes RNDR on an AArch64 host with the feature. `=+rand` (with the
    /// `=`) does not match inside `=+rdrand`.
    pub fn exposes_arm(&self) -> bool {
        let l = self.detail.to_ascii_lowercase();
        l.contains("target-feature=+rand") || l.contains("target-cpu=native")
    }
}

pub fn scan_cargo_toml(file: &str, src: &str) -> Vec<BuildConfigFinding> {
    let mut out = Vec::new();
    let Ok(val) = toml::from_str::<toml::Value>(src) else {
        return out;
    };
    // `[target.<triple>.rustflags]` or `[target.<cfg>.rustflags]`
    if let Some(target) = val.get("target").and_then(|v| v.as_table()) {
        for (_t, sub) in target {
            if let Some(flags) = sub.get("rustflags")
                && let Some(detail) = rustflags_mentions(flags)
            {
                out.push(BuildConfigFinding {
                    file: file.into(),
                    detail,
                });
            }
        }
    }
    // `[profile.*.rustflags]` is not stable cargo, but some crates set it.
    if let Some(profile) = val.get("profile").and_then(|v| v.as_table()) {
        for (_p, sub) in profile {
            if let Some(flags) = sub.get("rustflags")
                && let Some(detail) = rustflags_mentions(flags)
            {
                out.push(BuildConfigFinding {
                    file: file.into(),
                    detail,
                });
            }
        }
    }
    // Custom `[package.metadata.*]` blocks can carry instructions for build
    // tooling; we record any rustflags-shaped string mentioning rdrand.
    if let Some(detail) = recursive_rustflags_mention(&val) {
        out.push(BuildConfigFinding {
            file: file.into(),
            detail,
        });
    }
    dedup(out)
}

pub fn scan_cargo_config(file: &str, src: &str) -> Vec<BuildConfigFinding> {
    let mut out = Vec::new();
    let Ok(val) = toml::from_str::<toml::Value>(src) else {
        return out;
    };
    if let Some(build) = val.get("build")
        && let Some(flags) = build.get("rustflags")
        && let Some(detail) = rustflags_mentions(flags)
    {
        out.push(BuildConfigFinding {
            file: file.into(),
            detail,
        });
    }
    if let Some(target) = val.get("target").and_then(|v| v.as_table()) {
        for (_t, sub) in target {
            if let Some(flags) = sub.get("rustflags")
                && let Some(detail) = rustflags_mentions(flags)
            {
                out.push(BuildConfigFinding {
                    file: file.into(),
                    detail,
                });
            }
        }
    }
    dedup(out)
}

pub fn scan_build_rs(file: &str, src: &str) -> Vec<BuildConfigFinding> {
    let mut out = Vec::new();
    for line in src.lines() {
        let l = line.trim();
        if !l.contains("cargo:") {
            continue;
        }
        let lower = l.to_ascii_lowercase();
        if (lower.contains("rustc-flags") || lower.contains("rustc-cfg"))
            && (lower.contains("rdrand") || lower.contains("rdseed") || lower.contains("native"))
        {
            out.push(BuildConfigFinding {
                file: file.into(),
                detail: l.to_string(),
            });
        }
    }
    out
}

fn rustflags_mentions(v: &toml::Value) -> Option<String> {
    let s = match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Array(arr) => arr
            .iter()
            .filter_map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        _ => return None,
    };
    let lower = s.to_ascii_lowercase();
    if lower.contains("+rdrand")
        || lower.contains("+rdseed")
        || lower.contains("target-cpu=native")
        || lower.contains("target-feature=+rdrand")
        || lower.contains("target-feature=+rdseed")
        || lower.contains("target-feature=+rand")
    {
        Some(s)
    } else {
        None
    }
}

#[derive(Deserialize)]
struct _Unused {} // silence unused-deser warning for the toml crate

fn recursive_rustflags_mention(v: &toml::Value) -> Option<String> {
    fn walk(v: &toml::Value) -> Option<String> {
        match v {
            toml::Value::String(s) => {
                let lower = s.to_ascii_lowercase();
                if (lower.contains("rustflags") || lower.contains("target-feature"))
                    && (lower.contains("rdrand") || lower.contains("rdseed"))
                {
                    return Some(s.clone());
                }
                None
            }
            toml::Value::Table(t) => {
                for (_, sub) in t {
                    if let Some(s) = walk(sub) {
                        return Some(s);
                    }
                }
                None
            }
            toml::Value::Array(a) => {
                for x in a {
                    if let Some(s) = walk(x) {
                        return Some(s);
                    }
                }
                None
            }
            _ => None,
        }
    }
    walk(v)
}

fn dedup(mut v: Vec<BuildConfigFinding>) -> Vec<BuildConfigFinding> {
    v.sort_by(|a, b| (a.file.as_str(), a.detail.as_str()).cmp(&(b.file.as_str(), b.detail.as_str())));
    v.dedup_by(|a, b| a.file == b.file && a.detail == b.detail);
    v
}
