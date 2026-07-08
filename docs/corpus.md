# Corpus

The Rust crate corpus underlying the RDRAND prevalence study.

## Snapshot

- **Source.** `https://static.crates.io/db-dump.tar.gz`
- **Snapshot date.** 2026-05-13 (db-dump directory `2026-05-13-020017`)
- **db-dump sha256.**
  `db6bf2ac1b1217d627bcae1594d0f78e32ded593eb128a6fbf174efb2e71e974`
- **Crate selection rule.** For each crate, the most recently created
  non-yanked, non-prerelease (no `-` in the semver string) version.
- **Sha256 verification.** Every `.crate` tarball was verified against
  the `checksum` column from `versions.csv` at fetch time. Mismatches
  abort the fetch for that crate; none were observed.
- **Tooling.** `crawler` binary in `pipeline/crawler/`. Concurrency 8,
  User-Agent identifies the requester per crates.io's crawler policy.

## What is in the corpus

| Metric | Value |
|---|---|
| Crates in db-dump | 266,128 |
| Crates with at least one non-yanked stable version (targets) | 252,211 |
| Crate-versions successfully fetched | **252,202** |
| Total `.crate` tarball size on disk | 34 GB |
| Manifest rows (excluding header) | 252,202 |

The corpus, manifest, and snapshot pin live under
`$CRATES_SCRATCH_DIR` (default `~/scratch/crates-io/`) and are not
committed to this repository. They are reproducible from the snapshot
pin (`snapshot.toml`) plus the per-crate sha256s in `manifest.csv`.

## Limitations and exclusions

### Withdrawn crates (9, unrecoverable)

Nine crates appear in the db-dump metadata but their tarballs return
HTTP 403 from `static.crates.io`. These are crates.io tombstones — the
metadata persists but the source has been pulled (typically DMCA, owner
request, or policy enforcement). They are excluded from the corpus and
must be excluded from any per-crate-version classification:

```
go-updater                       0.1.0
merge-code                       0.1.0
psy_american                     0.2.7
psyfi-serum-dex-permissioned     0.5.11
psyoptions                       0.0.5
qemu-debugcon                    0.1.0
realistic_blas                   0.3.3
uconsole                         0.0.0
yue                              0.1.0
```

This is an irreducible 0.0036% measurement gap. None of the withdrawn
crates appear in the top-N-downloaded lists relevant to the study.

### Yanked-only crates

Crates whose only published versions are yanked are excluded. The
yank state is informative (a maintainer signal that the version should
not be used), and excluded versions are recorded for the discussion
section but not classified.

### Pre-release versions

Versions with `-` in the semver string (e.g. `1.0.0-rc.1`,
`0.1.0-alpha`) are excluded. Rationale: pre-releases are not pulled by
default by `cargo`, so they do not represent realistic downstream
exposure. A small number of crates publish *only* pre-releases and are
therefore absent from the corpus; this is consistent with the study's
focus on what end-users actually compile.

### Latest-only sampling

Only the most recent stable version of each crate is fetched, not all
versions ever published. A crate that previously had unconditional
RDRAND but has since added a CPUID gate appears in the corpus only in
its current, gated form. Historical analysis (e.g. when the
`getrandom` CPUID-removal `#[cfg]` first landed) is performed
separately by inspecting git history of named crates, not by sampling
older versions from the corpus.

### "Latest" by `created_at`, not by semver ordering

When a crate has multiple non-yanked stable branches under active
development (e.g. a `1.x` series and a backported `0.9.x`), the
crawler picks the version with the most recent `created_at` timestamp,
which can be the backport rather than the higher semver. This is rare
in practice (multi-branch maintenance is uncommon outside of a handful
of foundational crates) and is documented here as a known sampling
quirk. A semver-aware refinement is a candidate follow-up if the
classifier results indicate sensitivity to this choice.

### Non-crates.io code

Source vendored into a crate (e.g. `vendor/` directories shipping
third-party C or Rust), git-dependency-only crates not published to
crates.io, and private registries are out of scope. The Debian binary
scan in Paper 1 provides a partial cross-check for code outside
crates.io.

### Build-config exposure

Whether a crate is compiled with `-C target-feature=+rdrand` or
`-C target-cpu=native` is a build-site decision, not a property of the
crate source. The corpus permits detection of crates that *enable*
this via `.cargo/config.toml`, `Cargo.toml`, or `build.rs` rustflags
settings, but cannot quantify ambient host-side `RUSTFLAGS`
environment use. The Gentoo `CFLAGS=-march=native` precedent cited in
`problem.tex` is the closest available proxy for that population.

### Macro-generated RDRAND

Source-level static analysis on the unexpanded AST will miss
`RDRAND`/`RDSEED` usage produced by macro expansion. Mitigation: run
`cargo expand` on a stratified sub-sample (Section 4 of `PLAN.md`) and
report any classifier deltas.

## Reproducibility

To rebuild the corpus from scratch:

1. Fetch the db-dump at the pinned sha256 (or accept a current
   snapshot and re-pin).
2. Run `crawler fetch --concurrency 8`.
3. Confirm `manifest.csv` row count and the per-crate sha256s match
   the archived manifest.

The 9 withdrawn crates listed above will not re-appear; they are a
permanent exclusion as of this snapshot date.
