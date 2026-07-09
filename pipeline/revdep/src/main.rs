//! Reverse-dependency closure for the RDRAND prevalence study.
//!
//! Given the crates that carry a compile-time RDRAND source (and the mixed-use
//! wrapper libraries), compute every crate that transitively pulls one of them
//! in through *default-linked* dependency edges, then weight the closure by
//! downloads.
//!
//! Edge model (locked in PLAN.md / next.md step 3):
//!   * `kind = 0`      (normal deps only; build- and dev-deps excluded)
//!   * `optional = f`  (always-linked; feature-gated opt-in deps excluded)
//!   * dependent side resolved to each crate's *default* version, matching
//!     what `cargo` resolves by default.
//!
//! This is the tightest, most defensible "default-linked" closure. It does not
//! resolve semver `req` ranges or transitive feature unification; those are
//! documented threats to validity in corpus.md / the paper.

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Mixed-use wrapper libraries: they emit RDRAND with no in-crate gate, and
/// whether the resulting binary is rr-relevant depends on the downstream
/// consumer. Reported as a *separate* closure from the compile_time headline.
const WRAPPER_SEEDS: &[&str] = &["rdrand", "x86", "x86_64", "x64", "lokacore", "mc-rand"];

/// Special-case crates whose downstream tracing PLAN.md section 5 calls out.
const SPECIAL_CASES: &[&str] = &["getrandom", "ring", "rand", "rand_core", "aws-lc-rs"];

#[derive(Parser)]
#[command(about = "Reverse-dependency closure of RDRAND-carrying crates")]
struct Args {
    /// Scratch dir holding the extracted db-dump (defaults to
    /// $CRATES_SCRATCH_DIR or ~/scratch/crates-io).
    #[arg(long)]
    scratch: Option<PathBuf>,

    /// Per-crate classification CSV produced by the classifier.
    #[arg(long, default_value = "results/crates.csv")]
    crates_csv: PathBuf,

    /// Output directory for closure.csv and closure_summary.md.
    #[arg(long, default_value = "results")]
    out: PathBuf,

    /// Include optional (feature-gated) normal deps as edges. Off by default,
    /// which yields the tight "default-linked" lower bound; on yields the
    /// "could inherit if features enabled" upper bound.
    #[arg(long)]
    include_optional: bool,

    /// Which seed class to draw the headline closure from:
    /// `compile_time` (crates with a compile-time entropy source) or
    /// `build_config` (crates whose packaging force-enables the feature via
    /// rustflags). Selects the corresponding column of `--crates-csv`.
    #[arg(long, default_value = "compile_time")]
    seed_mode: String,

    /// Override the mixed-use wrapper-library seed list (comma-separated).
    /// Defaults to the x86 wrappers; pass the AArch64 register wrappers for an
    /// ARM run, or an empty string to disable the wrapper closure.
    #[arg(long)]
    wrappers: Option<String>,

    /// Prefix for output filenames (e.g. `arm_` -> `arm_closure.csv`), so
    /// x86/ARM and compile-time/build-config runs coexist.
    #[arg(long, default_value = "")]
    label: String,
}

/// Which seed class(es) reach a given crate. A crate can inherit from both.
#[derive(Clone, Copy, Default)]
struct Reach {
    compile_time: bool,
    wrapper: bool,
    /// BFS depth from the nearest seed (1 == direct dependent).
    depth: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let scratch = resolve_scratch(args.scratch)?;
    let data = dump_data_dir(&scratch)?;
    eprintln!("db-dump data: {}", data.display());

    // --- crate_id <-> name -------------------------------------------------
    let (id_to_name, name_to_id) = load_crates(&data.join("crates.csv"))?;
    eprintln!("crates: {}", id_to_name.len());

    // --- per-crate total downloads ----------------------------------------
    let downloads = load_downloads(&data.join("crate_downloads.csv"))?;
    let total_downloads: u128 = downloads.values().map(|&d| d as u128).sum();

    // --- default version_id -> owning crate_id ----------------------------
    // Only edges declared by a crate's default version participate.
    let default_ver_to_crate = load_default_versions(&data.join("default_versions.csv"))?;
    eprintln!("default versions: {}", default_ver_to_crate.len());

    // --- seeds -------------------------------------------------------------
    let wrapper_list: Vec<String> = match &args.wrappers {
        Some(s) => s
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        None => WRAPPER_SEEDS.iter().map(|s| s.to_string()).collect(),
    };
    let (ct_names, _wrap_names) = load_seed_names(&args.crates_csv, &args.seed_mode)?;
    let mut ct_seeds: HashSet<u32> = HashSet::new();
    let mut wrap_seeds: HashSet<u32> = HashSet::new();
    for n in &ct_names {
        if let Some(&id) = name_to_id.get(n.as_str()) {
            ct_seeds.insert(id);
        } else {
            eprintln!("warning: seed {n:?} not found in crates.csv");
        }
    }
    for n in &wrapper_list {
        if let Some(&id) = name_to_id.get(n.as_str()) {
            wrap_seeds.insert(id);
        } else {
            eprintln!("warning: wrapper seed {n:?} not found in crates.csv");
        }
    }
    eprintln!(
        "seeds: {} compile_time, {} wrapper",
        ct_seeds.len(),
        wrap_seeds.len()
    );

    // --- reverse adjacency: dependency crate_id -> [dependent crate_ids] ---
    let radj = build_reverse_adjacency(
        &data.join("dependencies.csv"),
        &default_ver_to_crate,
        args.include_optional,
    )?;
    let edge_count: usize = radj.values().map(|v| v.len()).sum();
    eprintln!(
        "normal edges ({}): {edge_count}",
        if args.include_optional {
            "incl. optional / upper bound"
        } else {
            "default-linked / lower bound"
        }
    );

    // --- multi-source reverse BFS -----------------------------------------
    let reach = reverse_bfs(&radj, &ct_seeds, &wrap_seeds);

    // --- write outputs -----------------------------------------------------
    // Label-prefixed, mode-suffixed filenames so x86/ARM and
    // lower-/upper-bound runs coexist.
    let label = &args.label;
    let suffix = if args.include_optional { "_maxbound" } else { "" };
    std::fs::create_dir_all(&args.out)?;
    write_closure_csv(
        &args.out.join(format!("{label}closure{suffix}.csv")),
        &reach,
        &id_to_name,
        &downloads,
        &ct_seeds,
        &wrap_seeds,
    )?;

    let summary = summarize(
        &reach,
        &radj,
        &id_to_name,
        &name_to_id,
        &downloads,
        total_downloads,
        &ct_seeds,
        &wrap_seeds,
        &ct_names,
        &wrapper_list,
        args.include_optional,
    );
    let summary_path = args.out.join(format!("{label}closure_summary{suffix}.md"));
    std::fs::write(&summary_path, &summary)?;
    eprintln!("wrote {}", summary_path.display());
    print!("{summary}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Loaders
// ---------------------------------------------------------------------------

fn load_crates(path: &Path) -> Result<(HashMap<u32, String>, HashMap<String, u32>)> {
    let mut rdr = reader(path)?;
    let headers = rdr.headers()?.clone();
    let id_i = col(&headers, "id")?;
    let name_i = col(&headers, "name")?;
    let mut id_to_name = HashMap::new();
    let mut name_to_id = HashMap::new();
    let mut rec = csv::StringRecord::new();
    while rdr.read_record(&mut rec)? {
        let id: u32 = rec[id_i].parse().context("crate id")?;
        let name = rec[name_i].to_string();
        name_to_id.insert(name.clone(), id);
        id_to_name.insert(id, name);
    }
    Ok((id_to_name, name_to_id))
}

fn load_downloads(path: &Path) -> Result<HashMap<u32, u64>> {
    let mut rdr = reader(path)?;
    let headers = rdr.headers()?.clone();
    let id_i = col(&headers, "crate_id")?;
    let dl_i = col(&headers, "downloads")?;
    let mut out = HashMap::new();
    let mut rec = csv::StringRecord::new();
    while rdr.read_record(&mut rec)? {
        let id: u32 = rec[id_i].parse()?;
        let dl: u64 = rec[dl_i].parse().unwrap_or(0);
        out.insert(id, dl);
    }
    Ok(out)
}

fn load_default_versions(path: &Path) -> Result<HashMap<u32, u32>> {
    let mut rdr = reader(path)?;
    let headers = rdr.headers()?.clone();
    let crate_i = col(&headers, "crate_id")?;
    let ver_i = col(&headers, "version_id")?;
    let mut out = HashMap::new();
    let mut rec = csv::StringRecord::new();
    while rdr.read_record(&mut rec)? {
        let crate_id: u32 = rec[crate_i].parse()?;
        let ver_id: u32 = rec[ver_i].parse()?;
        out.insert(ver_id, crate_id);
    }
    Ok(out)
}

/// Headline seed names come from the classifier output. `seed_mode`
/// `compile_time` selects the `category == compile_time` crates; `build_config`
/// selects `build_config_exposure == true` crates (packaging that
/// force-enables the entropy feature). Returns (seed_names, wrapper_names) --
/// the wrapper slot is unused by callers (they pass their own list).
fn load_seed_names(path: &Path, seed_mode: &str) -> Result<(Vec<String>, Vec<String>)> {
    let mut rdr = reader(path)?;
    let headers = rdr.headers()?.clone();
    let name_i = col(&headers, "name")?;
    let cat_i = col(&headers, "category")?;
    let bc_i = col(&headers, "build_config_exposure")?;
    let mut seeds = Vec::new();
    let mut rec = csv::StringRecord::new();
    while rdr.read_record(&mut rec)? {
        let hit = match seed_mode {
            "build_config" => &rec[bc_i] == "true",
            _ => &rec[cat_i] == "compile_time",
        };
        if hit {
            seeds.push(rec[name_i].to_string());
        }
    }
    // classifier writes crates.csv via a mutex-protected writer fed by a
    // parallel (rayon) loop, so row order (and therefore the order seeds
    // are encountered in here) varies run to run even though row content
    // doesn't. Sort so the "Seeds (N): ..." line in the summary is
    // actually deterministic, not just the underlying data.
    seeds.sort();
    Ok((seeds, Vec::new()))
}

/// Stream dependencies.csv and build reverse adjacency over default-linked
/// normal, non-optional edges.
///
/// In the db-dump, a dependencies row means: the version `version_id` declares
/// a dependency *on* crate `crate_id`. We keep the row only if `version_id` is
/// some crate's default version; the dependent is that owning crate.
fn build_reverse_adjacency(
    path: &Path,
    default_ver_to_crate: &HashMap<u32, u32>,
    include_optional: bool,
) -> Result<HashMap<u32, Vec<u32>>> {
    let mut rdr = reader(path)?;
    let headers = rdr.headers()?.clone();
    let dep_target_i = col(&headers, "crate_id")?;
    let kind_i = col(&headers, "kind")?;
    let optional_i = col(&headers, "optional")?;
    let version_i = col(&headers, "version_id")?;

    let mut radj: HashMap<u32, Vec<u32>> = HashMap::new();
    // Dedup identical (dependent -> target) pairs; a version can list the same
    // crate multiple times across targets/features.
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    let mut rec = csv::ByteRecord::new();
    while rdr.read_byte_record(&mut rec)? {
        if &rec[kind_i] != b"0" {
            continue; // normal deps only
        }
        if !include_optional && &rec[optional_i] != b"f" {
            continue; // always-linked only (lower bound)
        }
        let ver_id: u32 = atoi(&rec[version_i]);
        let Some(&dependent) = default_ver_to_crate.get(&ver_id) else {
            continue; // edge not declared by a default version
        };
        let target: u32 = atoi(&rec[dep_target_i]);
        if target == dependent {
            continue; // ignore self-edges
        }
        if seen.insert((dependent, target)) {
            radj.entry(target).or_default().push(dependent);
        }
    }
    Ok(radj)
}

// ---------------------------------------------------------------------------
// Closure
// ---------------------------------------------------------------------------

/// Multi-source reverse BFS. Seeds are the sources; every crate reached
/// (excluding the seeds themselves) is tagged with which seed class(es) reach
/// it and the shortest depth.
fn reverse_bfs(
    radj: &HashMap<u32, Vec<u32>>,
    ct_seeds: &HashSet<u32>,
    wrap_seeds: &HashSet<u32>,
) -> HashMap<u32, Reach> {
    let mut reach: HashMap<u32, Reach> = HashMap::new();
    // (crate_id, compile_time?, wrapper?, depth)
    let mut q: VecDeque<(u32, bool, bool, u32)> = VecDeque::new();

    // Enqueue direct dependents of every seed at depth 1.
    for (&seed, (ct, wrap)) in ct_seeds
        .iter()
        .map(|s| (s, (true, false)))
        .chain(wrap_seeds.iter().map(|s| (s, (false, true))))
    {
        if let Some(deps) = radj.get(&seed) {
            for &d in deps {
                if ct_seeds.contains(&d) || wrap_seeds.contains(&d) {
                    continue; // never count a seed as part of the closure
                }
                q.push_back((d, ct, wrap, 1));
            }
        }
    }

    while let Some((id, ct, wrap, depth)) = q.pop_front() {
        let entry = reach.entry(id).or_default();
        // Propagate only class flags that are newly set for this crate; if we
        // add no new information and depth is not an improvement, stop.
        let new_ct = ct && !entry.compile_time;
        let new_wrap = wrap && !entry.wrapper;
        let first_visit = entry.depth == 0;
        if first_visit {
            entry.depth = depth;
        }
        let mut changed = first_visit;
        if new_ct {
            entry.compile_time = true;
            changed = true;
        }
        if new_wrap {
            entry.wrapper = true;
            changed = true;
        }
        if !changed {
            continue;
        }
        // Push dependents-of-this-crate one level further out, carrying only
        // the class flags this node actually holds.
        if let Some(deps) = radj.get(&id) {
            for &d in deps {
                if ct_seeds.contains(&d) || wrap_seeds.contains(&d) {
                    continue;
                }
                q.push_back((d, entry.compile_time, entry.wrapper, depth + 1));
            }
        }
    }
    reach
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

fn write_closure_csv(
    path: &Path,
    reach: &HashMap<u32, Reach>,
    id_to_name: &HashMap<u32, String>,
    downloads: &HashMap<u32, u64>,
    ct_seeds: &HashSet<u32>,
    wrap_seeds: &HashSet<u32>,
) -> Result<()> {
    let _ = (ct_seeds, wrap_seeds);
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "name",
        "crate_id",
        "seed_class",
        "depth",
        "direct",
        "downloads",
    ])?;
    // Stable, deterministic ordering: seed_class, then downloads desc, name.
    let mut rows: Vec<(u32, &Reach)> = reach.iter().map(|(&id, r)| (id, r)).collect();
    rows.sort_by(|(a, ra), (b, rb)| {
        let da = downloads.get(a).copied().unwrap_or(0);
        let db = downloads.get(b).copied().unwrap_or(0);
        db.cmp(&da).then_with(|| a.cmp(b)).then(ra.depth.cmp(&rb.depth))
    });
    for (id, r) in rows {
        let class = match (r.compile_time, r.wrapper) {
            (true, true) => "both",
            (true, false) => "compile_time",
            (false, true) => "wrapper",
            (false, false) => continue,
        };
        let name = id_to_name.get(&id).map(String::as_str).unwrap_or("?");
        let dl = downloads.get(&id).copied().unwrap_or(0);
        wtr.write_record([
            name,
            &id.to_string(),
            class,
            &r.depth.to_string(),
            if r.depth == 1 { "true" } else { "false" },
            &dl.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn summarize(
    reach: &HashMap<u32, Reach>,
    radj: &HashMap<u32, Vec<u32>>,
    id_to_name: &HashMap<u32, String>,
    name_to_id: &HashMap<String, u32>,
    downloads: &HashMap<u32, u64>,
    total_downloads: u128,
    ct_seeds: &HashSet<u32>,
    wrap_seeds: &HashSet<u32>,
    ct_names: &[String],
    wrap_names: &[String],
    include_optional: bool,
) -> String {
    let total_crates = id_to_name.len();

    let dl = |id: &u32| downloads.get(id).copied().unwrap_or(0) as u128;

    // compile_time closure = any crate with the compile_time flag set.
    let ct_ids: Vec<u32> = reach
        .iter()
        .filter(|(_, r)| r.compile_time)
        .map(|(&id, _)| id)
        .collect();
    let wrap_ids: Vec<u32> = reach
        .iter()
        .filter(|(_, r)| r.wrapper)
        .map(|(&id, _)| id)
        .collect();
    let both = reach.values().filter(|r| r.compile_time && r.wrapper).count();

    let ct_dl: u128 = ct_ids.iter().map(&dl).sum();
    let wrap_dl: u128 = wrap_ids.iter().map(&dl).sum();

    let pct = |n: u128, d: u128| if d == 0 { 0.0 } else { 100.0 * n as f64 / d as f64 };

    let ct_direct = reach.values().filter(|r| r.compile_time && r.depth == 1).count();
    let wrap_direct = reach.values().filter(|r| r.wrapper && r.depth == 1).count();

    let mut s = String::new();
    if include_optional {
        s.push_str("# Reverse-dependency closure (upper bound)\n\n");
        s.push_str(
            "Include-optional closure (`kind=normal`, optional deps kept,\n\
             dependent resolved to each crate's default version). Counts every\n\
             crate that *could* inherit the source if the relevant feature is\n\
             enabled. Seeds excluded from their own closure. See\n\
             `pipeline/revdep` and PLAN.md section 5.\n\n",
        );
    } else {
        s.push_str("# Reverse-dependency closure (lower bound)\n\n");
        s.push_str(
            "Default-linked closure (`kind=normal`, `optional=false`, dependent\n\
             resolved to each crate's default version). Seeds excluded from their\n\
             own closure. See `pipeline/revdep` and PLAN.md section 5.\n\n",
        );
    }
    s.push_str(&format!(
        "Corpus: {total_crates} crates, {total_downloads} total downloads.\n\n",
    ));

    s.push_str("## Compile-time RDRAND closure (headline)\n\n");
    s.push_str(&format!("Seeds ({}): {}\n\n", ct_seeds.len(), ct_names.join(", ")));
    s.push_str("| Metric | Value |\n|---|---:|\n");
    s.push_str(&format!(
        "| Crates transitively pulling a compile_time source | {} |\n",
        ct_ids.len()
    ));
    s.push_str(&format!(
        "| Crate-count share of corpus | {:.3}% |\n",
        pct(ct_ids.len() as u128, total_crates as u128)
    ));
    s.push_str(&format!("| Direct dependents (depth 1) | {ct_direct} |\n"));
    s.push_str(&format!(
        "| Downloads-weighted share | {:.2}% |\n",
        pct(ct_dl, total_downloads)
    ));
    s.push_str(&format!("| Closure downloads | {ct_dl} |\n\n"));

    s.push_str("## Wrapper-library closure (reported separately)\n\n");
    s.push_str(&format!(
        "Seeds ({}): {}\n\n",
        wrap_seeds.len(),
        wrap_names.join(", ")
    ));
    s.push_str("| Metric | Value |\n|---|---:|\n");
    s.push_str(&format!("| Crates transitively pulling a wrapper | {} |\n", wrap_ids.len()));
    s.push_str(&format!(
        "| Crate-count share of corpus | {:.3}% |\n",
        pct(wrap_ids.len() as u128, total_crates as u128)
    ));
    s.push_str(&format!("| Direct dependents (depth 1) | {wrap_direct} |\n"));
    s.push_str(&format!(
        "| Downloads-weighted share | {:.2}% |\n",
        pct(wrap_dl, total_downloads)
    ));
    s.push_str(&format!("| Closure downloads | {wrap_dl} |\n"));
    s.push_str(&format!("| Crates in both closures | {both} |\n\n"));

    // Special-case direct dependents (PLAN section 5).
    s.push_str("## Special-case direct dependents\n\n");
    s.push_str("Direct (depth-1) dependent counts for the crates PLAN section 5 tracks.\n\n");
    s.push_str("| Crate | In corpus | Direct dependents |\n|---|---|---:|\n");
    for name in SPECIAL_CASES {
        match name_to_id.get(*name) {
            // Direct default-linked dependents = reverse-adjacency out-degree.
            Some(&id) => {
                let direct = radj.get(&id).map(|v| v.len()).unwrap_or(0);
                s.push_str(&format!("| {name} | yes | {direct} |\n"));
            }
            None => s.push_str(&format!("| {name} | no | - |\n")),
        }
    }
    s.push('\n');

    // Top 15 highest-download crates in the compile_time closure.
    s.push_str("## Top compile_time-closure crates by downloads\n\n");
    let mut top: Vec<u32> = ct_ids.clone();
    top.sort_by_key(|id| std::cmp::Reverse(dl(id)));
    s.push_str("| Crate | Downloads | Depth |\n|---|---:|---:|\n");
    for id in top.iter().take(15) {
        let name = id_to_name.get(id).map(String::as_str).unwrap_or("?");
        let depth = reach.get(id).map(|r| r.depth).unwrap_or(0);
        s.push_str(&format!("| {name} | {} | {depth} |\n", dl(id)));
    }
    s.push('\n');
    s
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn reader(path: &Path) -> Result<csv::Reader<std::fs::File>> {
    csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .with_context(|| format!("opening {}", path.display()))
}

fn col(headers: &csv::StringRecord, name: &str) -> Result<usize> {
    headers
        .iter()
        .position(|h| h == name)
        .ok_or_else(|| anyhow!("column {name:?} not found"))
}

/// Fast base-10 parse of an ASCII byte slice; non-digits terminate.
fn atoi(b: &[u8]) -> u32 {
    let mut n: u32 = 0;
    for &c in b {
        if c.is_ascii_digit() {
            n = n * 10 + (c - b'0') as u32;
        } else {
            break;
        }
    }
    n
}

fn resolve_scratch(over: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = over {
        return Ok(p);
    }
    if let Ok(env) = std::env::var("CRATES_SCRATCH_DIR") {
        return Ok(PathBuf::from(env));
    }
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join("scratch").join("crates-io"))
}

/// Find `<scratch>/db-dump/<dated>/data`, choosing the newest dated dir.
fn dump_data_dir(scratch: &Path) -> Result<PathBuf> {
    let root = scratch.join("db-dump");
    let mut best: Option<PathBuf> = None;
    for entry in std::fs::read_dir(&root)
        .with_context(|| format!("reading {}", root.display()))?
        .flatten()
    {
        let data = entry.path().join("data");
        if data.join("crates.csv").is_file() {
            // Lexical max of the dated dir name == newest snapshot.
            if best.as_ref().is_none_or(|b| entry.path() > *b) {
                best = Some(entry.path());
            }
        }
    }
    best.map(|p| p.join("data"))
        .ok_or_else(|| anyhow!("no db-dump/<date>/data/crates.csv under {}", root.display()))
}
