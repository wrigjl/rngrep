# Corpus

The Rust crate corpus underlying the RDRAND prevalence study.

## Snapshot

- **Source.** `https://static.crates.io/db-dump.tar.gz`
- **Snapshot date.** 2026-07-08 (db-dump directory `2026-07-08-020028`)
- **db-dump sha256.**
  `af2a04c11d6fbed99307167d44d28c3360090c8eae186f5bc818cf42f8b0bf36`
- **Crate selection rule.** For each crate, the most recently created
  non-yanked, non-prerelease (no `-` in the semver string) version whose
  declared license permits unmodified redistribution (see "License-based
  target filtering" below).
- **Sha256 verification.** Every `.crate` tarball was verified against
  the `checksum` column from `versions.csv` at fetch time. Mismatches
  abort the fetch for that crate; none were observed.
- **Tooling.** `crawler` binary in `pipeline/crawler/`. Concurrency 8,
  User-Agent identifies the requester per crates.io's crawler policy.

## What is in the corpus

| Metric | Value |
|---|---|
| Crates in db-dump | 295,947 |
| Redistributably-licensed targets (post license filter) | 270,771 |
| License-excluded targets (see "License-based target filtering") | 9,729 |
| Crate-versions successfully fetched | **270,761** |
| Total `.crate` tarball size on disk | 37 GB |
| Manifest rows (excluding header) | 270,761 |

These figures describe the canonical corpus fetched for the paper
(snapshot 2026-07-08). Unlike the earlier 2026-05-13 pass (retained,
superseded, under `results_2026-05-13/`), this corpus **is**
license-filtered: the license-based target filter (see below) runs
*during* target selection, so a crate-version whose declared license
does not permit unmodified redistribution is never fetched. The
population is therefore "crates.io crates with a redistributable
license," and the corpus is redistributable by construction. All of the
paper's analysis numbers (classifier/revdep) are computed over this
270,761-crate corpus.

The corpus, manifest, and snapshot pin live under
`$CRATES_SCRATCH_DIR` (default `~/scratch/crates-io/`) and are not
committed to this repository. They are reproducible from the snapshot
pin (`snapshot.toml`) plus the per-crate sha256s in `manifest.csv`.

## Limitations and exclusions

### Withdrawn crates (10, unrecoverable)

Ten crates appear in the db-dump metadata but their tarballs return
HTTP 403 from `static.crates.io`. These are crates.io tombstones — the
metadata persists but the source has been pulled (typically DMCA, owner
request, or policy enforcement). They are excluded from the corpus and
must be excluded from any per-crate-version classification:

```
carbono                          0.1.1
encryption                       0.1.0
fleischwolf                      0.16.0
fleischwolf-asr                  0.16.0
fleischwolf-cli                  0.16.0
fleischwolf-core                 0.16.0
fleischwolf-pdf                  0.16.0
njoy-outram-park-fork            0.0.1
oauth-certs                      0.6.0
voice-stream                     0.4.0
```

This is an irreducible 0.0037% measurement gap. None of the withdrawn
crates appear in the top-N-downloaded lists relevant to the study.

### Corpus reproduction drift (illustrative, measured on the 2026-05-13 corpus)

The withdrawn-crate list above is the count *at pin time*. It is **not**
a stable prediction of what a later reproduction attempt will see: the
unrecoverable fraction grows with elapsed time since the pin. The
canonical 2026-07-08 corpus was fetched ~1 day after its pin, so its own
drift is currently negligible (the 10 tombstones above) — but the effect
is real and the following measurement, from the earlier 2026-05-13 pass,
quantifies it and applies equally to this pin going forward.

A full re-fetch of the 2026-05-13 corpus performed 8 weeks after that
pin recovered 251,209 of the 252,202 manifest rows, each verified
byte-identical (name, version, and SHA-256 all match) against
`results_2026-05-13/manifest.csv.gz`. The remaining 993 rows (0.39%, vs.
the 9/0.0036% at that snapshot's pin time) could not be reproduced:

- **988** now return HTTP 403. Only 8 of that snapshot's 9 tombstones
  are among these; the other ~980 are newly withdrawn since the pin —
  crates.io's tombstone/policy-enforcement population keeps growing
  after a snapshot is taken, it is not fixed at pin time.
- **14** return HTTP 200 but with content whose SHA-256 no longer
  matches the pinned checksum, including `qemu-debugcon 0.1.0` — one
  of that snapshot's 9 tombstones, which has since
  started serving different (still-mismatched, still non-403) content.
  This shows even an already-tombstoned entry's served bytes are not
  guaranteed stable over time.
- 11 HTTP 500s and 4 body-decode errors were also observed on the
  first pass but all succeeded on an immediate retry, confirming they
  were transient network faults rather than genuine unavailability.

Practical implication for reproducers: expect the unrecoverable/
mismatched fraction to be larger than the tombstone count at pin time,
and to grow with elapsed time since the snapshot. A gap in that range
should be treated as expected drift, not a broken pipeline. Whether any
of the
993 affected crate/version pairs would have altered the paper's
RDRAND/RNDR classification counts is checked separately once the
classifier has run on the reproduced corpus.

### License-based target filtering

`crawler fetch`'s target selection
(`pipeline/crawler/src/fetch.rs::pick_latest`) only selects a
crate-version whose declared `license` string (from the pinned
db-dump's `versions.csv`) permits unmodified redistribution. A crate
failing this check is never fetched at all -- this is a target-
selection filter, not a post-hoc deletion from an already-fetched
corpus. The check (`pipeline/crawler/src/license.rs`) parses the
license as a real SPDX expression (via the `spdx` crate, so `AND`/`OR`/
`WITH` compound expressions are handled correctly, not just string
matching), after normalizing three known non-strict Rust-ecosystem
conventions that would otherwise fail strict parsing:

- `/`-separated lists, with or without spaces (`"MIT/Apache-2.0"`,
  `"MIT / Apache-2.0"`) -> `"MIT OR Apache-2.0"`
- lowercase connectives (`"MIT or Apache-2.0"`) -> uppercase
- the deprecated trailing `+` ("or later") suffix (`"GPL-3.0+"`) ->
  the modern `-or-later` suffix

A license is accepted if every term in the (possibly normalized)
expression resolves to a real SPDX license id -- this includes
copyleft licenses (GPL/LGPL/AGPL family: redistributable, just with
reciprocal conditions), not only permissive ones. It is rejected if
any term fails to resolve: blank, crates.io's `"non-standard"`
placeholder (used when `Cargo.toml` points at a `license-file` instead
of an SPDX string -- there is no string to parse), a bare
`LicenseRef-*` term with no open fallback (e.g. `"LicenseRef-
Proprietary"`), or any other unparseable string.

Measured against the 2026-07-08 snapshot's `versions.csv`, this filter
excludes **9,729** crate-versions (about 3.5% of the pre-filter target
population of 280,500), leaving the 270,771 redistributable targets in
the table above. The excluded set breaks down as 9,005 `"non-standard"`
placeholders, 502 bare `LicenseRef-*` terms with no open fallback (e.g.
`"LicenseRef-Proprietary"`), 213 blank, and 9 otherwise-unparseable SPDX
strings. None of the paper's 49 named result crates (the x86/ARM
in-source detections) are affected; every one of them has a clean,
resolvable SPDX license. Excluded crate-versions and their reasons are
written to `license_exclusions.csv` in the scratch directory alongside
`manifest.csv` for every fetch run, so the exclusion is auditable rather
than silent.

**This filter is part of the canonical corpus's definition** — it runs
during target selection, so the paper's 270,761-crate corpus is exactly
the redistributably-licensed population, and the archived source can be
mirrored without a rights problem. The earlier, superseded 2026-05-13
pass (`results_2026-05-13/`) predates the filter and was **not**
license-filtered; that is one reason its source corpus is not
redistributable and is retained only as derived results (see
`results_2026-05-13/README.md`).

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

The 10 withdrawn crates listed above are a permanent exclusion as of
this snapshot date, but they are a floor, not a ceiling: additional
crates can become unavailable, and previously-tombstoned crates can
have their served content change, between the snapshot date and a
later reproduction attempt. See "Corpus reproduction drift" above for
an empirical measurement (993 affected crates 8 weeks after the earlier
2026-05-13 pin). Diff `manifest.csv` against the shipped
`results/manifest.csv.gz` on `name,version,sha256` to confirm the rows
you *do* have match byte-for-byte; do not expect an exact 270,761-row
count.
