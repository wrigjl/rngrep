use crate::ast::{self, Finding, Isa, Kind};
use crate::buildconfig::{self, BuildConfigFinding};
use crate::extract::{self, EntryKind};
use crate::outofscope::{self, Detection};
use crate::prefilter;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize, Clone)]
pub struct CallsiteRow {
    pub name: String,
    pub version: String,
    pub file: String,
    pub line: usize,
    pub kind: String,
    pub detail: String,
    /// `;`-joined cfg attributes from enclosing items (outermost first).
    pub cfg_gates: String,
    pub in_test_or_bench: bool,
    pub runtime_gated: bool,
    pub compile_gate_present: bool,
    pub target_feature_attr: bool,
    pub is_out_of_scope: bool,
    pub out_of_scope_reason: String,
    pub category: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SummaryRow {
    pub name: String,
    pub version: String,
    pub num_callsites: usize,
    pub num_intrinsic: usize,
    pub num_inline_asm: usize,
    pub n_runtime: usize,
    pub n_compile_gate: usize,
    pub n_unconditional: usize,
    pub n_out_of_scope: usize,
    pub n_in_test: usize,
    /// Worst-case non-test category across the crate.
    pub category: String,
    pub build_config_exposure: bool,
    pub build_config_detail: String,
    pub is_out_of_scope: bool,
    pub out_of_scope_reason: String,
    pub out_of_scope_signals: String,
    pub parse_failures: usize,
}

pub struct CrateReport {
    /// x86 (RDRAND/RDSEED) call sites and crate summary.
    pub callsites: Vec<CallsiteRow>,
    pub summary: SummaryRow,
    /// AArch64 (RNDR/RNDRRS) call sites and crate summary, reported into
    /// separate `arm_*` output files so the x86 dataset is unchanged.
    pub arm_callsites: Vec<CallsiteRow>,
    pub arm_summary: SummaryRow,
}

pub fn classify_crate(name: &str, version: &str, tarball: &Path) -> Result<CrateReport> {
    let entries = extract::read_crate(tarball)?;

    // Pass 1: gather Cargo.toml contents for SGX detection and the
    // build-config scan. SGX detection must happen before we classify
    // call sites because it influences the per-call-site category.
    let cargo_tomls: Vec<(String, String)> = entries
        .iter()
        .filter(|e| matches!(e.kind, EntryKind::CargoToml))
        .filter_map(|e| {
            std::str::from_utf8(&e.contents)
                .ok()
                .map(|s| (e.path.clone(), s.to_string()))
        })
        .collect();
    let cargo_toml_refs: Vec<(&str, &str)> = cargo_tomls
        .iter()
        .map(|(p, s)| (p.as_str(), s.as_str()))
        .collect();
    let oos = outofscope::detect(name, &cargo_toml_refs);

    // Pass 2: classify Rust sources and run the build-config scan. Findings
    // are split by ISA: x86 (RDRAND/RDSEED) and AArch64 (RNDR/RNDRRS).
    let mut callsites: Vec<CallsiteRow> = Vec::new();
    let mut arm_callsites: Vec<CallsiteRow> = Vec::new();
    let mut build_findings: Vec<BuildConfigFinding> = Vec::new();
    let mut parse_failures = 0usize;

    for e in &entries {
        match e.kind {
            EntryKind::Rust => {
                if !prefilter::looks_interesting(&e.contents) {
                    continue;
                }
                let src = match std::str::from_utf8(&e.contents) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                match ast::analyze(&e.path, src) {
                    Some(findings) => {
                        for f in findings {
                            let isa = f.isa;
                            let row = to_row(name, version, f, &oos);
                            match isa {
                                Isa::X86 => callsites.push(row),
                                Isa::Arm => arm_callsites.push(row),
                            }
                        }
                    }
                    None => {
                        parse_failures += 1;
                    }
                }
            }
            EntryKind::CargoToml => {
                if let Ok(s) = std::str::from_utf8(&e.contents) {
                    build_findings.extend(buildconfig::scan_cargo_toml(&e.path, s));
                }
            }
            EntryKind::CargoConfig => {
                if let Ok(s) = std::str::from_utf8(&e.contents) {
                    build_findings.extend(buildconfig::scan_cargo_config(&e.path, s));
                }
            }
            EntryKind::BuildRs => {
                if let Ok(s) = std::str::from_utf8(&e.contents) {
                    build_findings.extend(buildconfig::scan_build_rs(&e.path, s));
                }
            }
        }
    }

    // Split build-config exposure by ISA: x86 gets `+rdrand`/native (the
    // existing behavior, so `crates.csv` is unchanged); ARM gets
    // `+rand`/native. `target-cpu=native` is ISA-neutral and appears on both.
    let x86_build: Vec<BuildConfigFinding> =
        build_findings.iter().filter(|f| f.exposes_x86()).cloned().collect();
    let arm_build: Vec<BuildConfigFinding> =
        build_findings.iter().filter(|f| f.exposes_arm()).cloned().collect();
    let summary = summarize(
        name,
        version,
        &callsites,
        &x86_build,
        &oos,
        parse_failures,
    );
    // Parse failures are an x86/ARM-shared property of the file and reported
    // on the x86 side only, to avoid double counting.
    let arm_summary = summarize(name, version, &arm_callsites, &arm_build, &oos, 0);
    Ok(CrateReport {
        callsites,
        summary,
        arm_callsites,
        arm_summary,
    })
}

fn to_row(name: &str, version: &str, f: Finding, oos: &Detection) -> CallsiteRow {
    let isa = f.isa;
    let compile_gate_present = f.cfg_gates.iter().any(|g| cfg_is_compile_gate(g, isa));
    let compile_signal = compile_gate_present || f.target_feature_attr;
    // Precedence: source-level signals win over out-of-scope detection.
    // A crate that is SGX-/kernel-/TEE-targeted *and* has source-level
    // gating keeps the source-level classification because the source
    // is the more informative description. Only call sites that would
    // otherwise be `unconditional` fall through to `out_of_scope`.
    let category = if f.runtime_gated && !compile_signal {
        "runtime"
    } else if compile_signal {
        "compile_time"
    } else if oos.is_out_of_scope {
        "out_of_scope"
    } else {
        "unconditional"
    };
    CallsiteRow {
        name: name.into(),
        version: version.into(),
        file: f.file,
        line: f.line,
        kind: match f.kind {
            Kind::Intrinsic => "intrinsic".into(),
            Kind::InlineAsm => "inline_asm".into(),
        },
        detail: f.detail,
        cfg_gates: f.cfg_gates.join(";"),
        in_test_or_bench: f.in_test_or_bench,
        runtime_gated: f.runtime_gated,
        compile_gate_present,
        target_feature_attr: f.target_feature_attr,
        is_out_of_scope: oos.is_out_of_scope,
        out_of_scope_reason: oos.reason.clone(),
        category: category.into(),
    }
}

/// Returns true if a cfg attribute string contains a predicate that
/// indicates the call site is gated on the target feature being present at
/// compile time (i.e. the runtime CPUID gate has been removed). This is
/// the dangerous category for our study.
fn cfg_is_compile_gate(s: &str, isa: Isa) -> bool {
    let lower = s.to_ascii_lowercase().replace([' ', '\t', '\n'], "");
    // SGX target gates elide the runtime check on either ISA.
    let sgx = lower.contains("target_env=\"sgx\"") || lower.contains("target_os=\"sgx\"");
    let feature = match isa {
        Isa::X86 => {
            lower.contains("target_feature=\"rdrand\"")
                || lower.contains("target_feature=\"rdseed\"")
        }
        // AArch64 FEAT_RNG compile-time gate. `="rand"` (with the closing
        // quote) does not match inside `="rdrand"`.
        Isa::Arm => lower.contains("target_feature=\"rand\""),
    };
    sgx || feature
}

fn summarize(
    name: &str,
    version: &str,
    callsites: &[CallsiteRow],
    build_findings: &[BuildConfigFinding],
    oos: &Detection,
    parse_failures: usize,
) -> SummaryRow {
    let mut num_intrinsic = 0;
    let mut num_inline_asm = 0;
    let mut n_runtime = 0;
    let mut n_compile_gate = 0;
    let mut n_unconditional = 0;
    let mut n_out_of_scope = 0;
    let mut n_in_test = 0;
    let mut worst: Option<&str> = None;
    for c in callsites {
        if c.kind == "intrinsic" {
            num_intrinsic += 1;
        } else {
            num_inline_asm += 1;
        }
        if c.in_test_or_bench {
            n_in_test += 1;
            continue;
        }
        match c.category.as_str() {
            "runtime" => n_runtime += 1,
            "compile_time" => n_compile_gate += 1,
            "unconditional" => n_unconditional += 1,
            "out_of_scope" => n_out_of_scope += 1,
            _ => {}
        }
        worst = Some(merge_worst(worst, &c.category));
    }
    let category = worst.unwrap_or("none").to_string();
    let build_config_exposure = !build_findings.is_empty();
    let detail = build_findings
        .iter()
        .map(|f| format!("{}::{}", f.file, f.detail))
        .collect::<Vec<_>>()
        .join(" | ");

    SummaryRow {
        name: name.into(),
        version: version.into(),
        num_callsites: callsites.len(),
        num_intrinsic,
        num_inline_asm,
        n_runtime,
        n_compile_gate,
        n_unconditional,
        n_out_of_scope,
        n_in_test,
        category,
        build_config_exposure,
        build_config_detail: detail,
        is_out_of_scope: oos.is_out_of_scope,
        out_of_scope_reason: oos.reason.clone(),
        out_of_scope_signals: oos.signals.join(";"),
        parse_failures,
    }
}

fn merge_worst<'a>(cur: Option<&'a str>, new: &'a str) -> &'a str {
    // Ordered by "how concerning for rr's recordable-Linux-userspace
    // scope": unconditional and compile_time are both dangerous;
    // runtime is safe; sgx_out_of_scope is outside the scope rr
    // operates in at all. unconditional outranks compile_time because
    // the latter usually implies a deliberate design choice (`getrandom`,
    // `safe_arch`); the former is more often unguarded use that
    // surprises an `rr` user.
    fn rank(c: &str) -> u8 {
        match c {
            "unconditional" => 4,
            "compile_time" => 3,
            "runtime" => 2,
            "out_of_scope" => 1,
            _ => 0,
        }
    }
    match cur {
        Some(c) if rank(c) >= rank(new) => c,
        _ => new,
    }
}
