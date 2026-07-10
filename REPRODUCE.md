# Reproducing rngrep

This document tells a researcher how to reproduce **every number** in the paper
*"Mining RDRAND: A Dataset of Hardware-Entropy Instruction Use Across the Rust
Crate Ecosystem"* from the pinned crates.io snapshot, and — for each number —
which file and command produces it.

Read it top to bottom once before starting: the corpus fetch is large (~37 GB
of `.crate` tarballs plus a ~5 GB db-dump — roughly **40 GB total**) and the
db-dump pin has a prerequisite (§2) that you must satisfy *before* the closure
numbers can be reproduced. The classifier **streams** each tarball through a
gzip decoder in memory and never extracts the corpus to disk, so there is no
large decompressed working set; binary verification (Stage 4) uses a few GB of
ephemeral `cargo` build scratch for the ~49 in-source crates only.

---

## 1. What is reproducible, and how

The pipeline has two independent inputs, with different reproduction stories:

| Input | Pins it | Reproducible? |
|---|---|---|
| The `.crate` source corpus (270,761 tarballs) | `results/manifest.csv.gz` (crate→version→sha256) + `snapshot.toml` | **Byte-for-byte, for tarballs still served.** Each tarball is re-fetchable from `static.crates.io` and verified against its recorded SHA-256. The unrecoverable/altered fraction grows with elapsed time since the pin (crates.io continues removing/altering crates after a snapshot is taken) — see `docs/corpus.md` § Corpus reproduction drift for a measured example (0.39% 8 weeks after the earlier 2026-05-13 pin; the current pin was fetched ~1 day out, so 0.0037% at paper-shipping time). Every row you *do* recover should still match the shipped manifest exactly. |
| The crates.io **db-dump** (metadata, downloads, reverse-dependency edges) | `snapshot.toml` (`sha256`) | **Only from an archived copy.** crates.io serves *only the latest* dump at a stable URL and does not retain historical dumps — see §2. |

Everything downstream is deterministic given those two inputs:

- **Classifier** (source scan) — pure function of the corpus. Same tarballs →
  same `crates.csv` / `callsites.csv`.
- **Revdep** (closures) — pure function of the db-dump. Same dump → same
  `closure*.csv`.
- **Binary verification** — deterministic given a fixed `rustc` (pinned to
  1.95.0 in `rust-toolchain.toml`); a different compiler may inline or fold
  differently, so the toolchain pin is load-bearing here.

> **The one hard prerequisite:** to reproduce the closure/download numbers you
> must obtain the *pinned* db-dump (§2). Without it you can still reproduce the
> entire source-classification half of the paper from the manifest alone.

---

## 2. The db-dump pin (read before Stage 1)

`snapshot.toml` pins the db-dump used for the paper:

```
url    = https://static.crates.io/db-dump.tar.gz
sha256 = af2a04c11d6fbed99307167d44d28c3360090c8eae186f5bc818cf42f8b0bf36
fetched_at = 2026-07-09T00:08:55Z      # dump directory: 2026-07-08-020028
```

That URL always serves **the current day's** dump. crates.io does **not**
publish historical dumps at a retrievable URL. So a fresh `crawler db-dump`
today downloads a *different* population and every closure number will differ.

To reproduce faithfully, obtain the archived pinned dump and stage it:

The pinned `db-dump.tar.gz` (1.44 GB) is archived in two places:

- **Zenodo** — the copy of record, alongside the rest of the artifact (see the
  DOI in `CITATION.cff` / the paper's Data Availability section). Cite/link this
  one.
- **GitHub Release** — a convenience mirror attached to the tagged release of
  this repo, for reproducers who already have the repo checked out.

Both are byte-identical (the dump is immutable and SHA-pinned); grab either:

```bash
export CRATES_SCRATCH_DIR="$HOME/scratch/crates-io"   # or wherever you have ~150 GB
mkdir -p "$CRATES_SCRATCH_DIR"
cp /path/to/archived/db-dump.tar.gz "$CRATES_SCRATCH_DIR/db-dump.tar.gz"
# or, from the GitHub Release:
#   gh release download <tag> -R wrigjl/rngrep -p db-dump.tar.gz \
#       -D "$CRATES_SCRATCH_DIR"
```

`crawler db-dump --expect-sha <pinned sha256>` then verifies the staged archive
byte-for-byte and **aborts** if it does not match the pin (so an accidental
fresh fetch cannot silently corrupt the run). `reproduce.sh` passes
`--expect-sha` automatically from `snapshot.toml`.

If you only want the source-classification results and accept a *current*
snapshot for the closure, run `crawler db-dump` with no `--expect-sha` and
re-pin: the numbers will then describe today's ecosystem, not the paper's.

> **Archival responsibility (author).** The Zenodo deposit **must** include
> `db-dump.tar.gz` (1.44 GB) in addition to `manifest.csv.gz`, or the closure
> numbers are not reproducible by a third party. Upload it to Zenodo as an
> explicit file: the GitHub→Zenodo integration snapshots only the repo tarball
> and captures neither release assets nor LFS objects. The classifier corpus
> does not need archiving — the manifest reconstructs it from
> `static.crates.io`. The dump is deliberately **not** committed to git (1.44 GB
> blows GitHub's 100 MB file limit, and LFS its free quota).

---

## 3. Prerequisites

| Need | For | Install |
|---|---|---|
| `rustup` + rustc **1.95.0** | everything | `rust-toolchain.toml` auto-selects 1.95.0 in this directory; rustup installs it (and the `x86_64-unknown-linux-gnu` + `x86_64-fortanix-unknown-sgx` targets listed there) on first `cargo` invocation. |
| ~40 GB scratch + network | corpus | ~37 GB `.crate` tarballs + ~5 GB db-dump (tarballs are streamed, not extracted to disk); access to `static.crates.io`. A few GB more of ephemeral `cargo` scratch for Stage 4. |
| GNU `objdump` (binutils) | binary verification | `apt install binutils` / present on most Linux hosts. `llvm-objdump` works too. |
| A native **x86_64** Linux host | x86 binary verification | disassembly only — no RDRAND-capable or privileged hardware required. |
| A native **aarch64** host | ARM binary verification | same; disassembly only. |
| `nightly` toolchain + `cargo-expand` | recall / macro-expansion validation only | `rustup toolchain install nightly && cargo install cargo-expand`. Invoked as `cargo +nightly expand`. |
| Python 3 | recall validation scripts (`pipeline/audit/*.py`) | stdlib only. |
| DuckDB (optional) | re-querying the shipped CSVs | `brew install duckdb` / static binary. Any CSV tool works. |

The corpus fetch identifies itself with a contact-bearing User-Agent per
crates.io's crawler policy (`pipeline/crawler/src/config.rs`). If you re-fetch,
change that UA to identify *you*.

---

## 4. Stage-by-stage

`reproduce.sh` drives stages 0–3 end to end; stage 4 is manual and per-ISA.
Run from the repo root with the pinned toolchain active (automatic in this
directory).

```bash
export CRATES_SCRATCH_DIR="$HOME/scratch/crates-io"
./reproduce.sh              # build → (corpus) → classify → closures
SKIP_CORPUS=1 ./reproduce.sh   # reuse an already-fetched corpus scratch
```

**Stage 0 — build.** `cargo build --release` for `crawler`, `classifier`,
`revdep`.

**Stage 1 — corpus.** Stage the pinned `db-dump.tar.gz` (§2), then:
```bash
crawler db-dump --expect-sha <pinned sha256>   # verifies the staged dump
crawler fetch                                  # ~37 GB of .crate tarballs
```
`fetch` is idempotent/append-only: it verifies each tarball against the
per-version checksum in the dump's `versions.csv` (column `tar_sha256` in
this dump; the crawler also accepts the older `checksum` name) and records
`name,version,sha256,size_bytes,fetched_at` in `manifest.csv`. Target
selection also excludes crate-versions whose declared license doesn't
permit unmodified redistribution (**9,729**, about 3.5% of targets on the
2026-07-08 snapshot); excluded crate-versions and why are written to
`license_exclusions.csv` alongside `manifest.csv`. See `docs/corpus.md`
§ License-based target filtering — this filter runs during target
selection, so the paper's **270,761-crate corpus is license-filtered by
construction** (unlike the earlier, superseded 2026-05-13 pass). If a
first pass logs `WARN`s, re-run `crawler fetch` once — HTTP 500s and
body-decode errors are usually transient and resolve on retry; HTTP 403s
and checksum mismatches will not. Don't expect exactly **270,761** rows:
crates.io keeps removing/altering crates after the pin date, so a
reproduction run further from 2026-07-08 will recover fewer (see
`docs/corpus.md` § Corpus reproduction drift for a measured example on the
earlier snapshot: 251,209 rows, 8 weeks post-pin). Confirm reproducibility
instead by diffing your `manifest.csv` against the shipped
`results/manifest.csv.gz` on `name,version,sha256` — every row you do have
should match exactly; a shrinking row count is expected drift, not a broken
pipeline.

**Stage 2 — classify.** `classifier --out results/` writes `crates.csv`,
`callsites.csv`, and the `arm_*` variants over the whole corpus.

**Stage 3 — closures.** Six `revdep` invocations (x86/ARM ×
lower/upper/build-config); see `reproduce.sh` for the exact flags. Outputs:
`closure*.csv`, `arm_closure*`, `bc_closure*`, `arm_bc_closure*` and their
`*_summary.md`.

**Stage 4 — binary verification (manual, per-ISA native host).** Disassembly
only; no privileged hardware. The driver scripts (`pipeline/verify/*.sh`,
`pipeline/audit/expand_run.sh`) were written for the authors' build hosts and
use host-local scratch paths (`$HOME/rrv`, `$HOME/rdrand-expand`) and env
overrides (`TC`, `BASE`, `CAND`, `OUT`) — set those to your paths and stage the
candidate CSVs (`pipeline/verify/*_candidates.csv`) into `$BASE` before running.
```bash
# x86_64 host:
TC=1.95.0 BASE=$HOME/rrv bash pipeline/verify/verify.sh
TC=1.95.0 BASE=$HOME/rrv bash pipeline/verify/refine.sh
# aarch64 host:
TC=1.95.0 BASE=$HOME/rrv bash pipeline/verify/arm_verify.sh
```
Outputs: `verify_results.csv`, `verify_refine.csv`, `arm_verify_results.csv`;
emit/no-emit soundness for this snapshot is narrated in `audit_results.md`
(the `verify_summary.md` / `arm_verify_summary.md` prose rollups exist only
for the earlier 2026-05-13 pass, under `results_2026-05-13/`).

**Recall validation (optional, nightly).** `pipeline/audit/expand_run.sh`
(needs `cargo +nightly expand`) plus the `corpus_grep*.py` /
`candidate_source_scan.py` scripts reproduce the exhaustive macro-expansion
recall check; narrated in `expand_validation.md`.

---

## 5. Number → provenance map

Each headline number in the paper, and how to regenerate it from the shipped
dataset. DuckDB one-liners query the released CSVs **in place** (no import); run
them from `results/`.

| Paper / README figure | Source of truth | Regenerate |
|---|---|---|
| 295,947 crates in db-dump; 356,393,391,514 total downloads | db-dump | header of `results/closure_summary.md` (from Stage 3) |
| 270,771 target crates; **270,761** fetched; 10 unrecoverable (403) | corpus | `zcat results/manifest.csv.gz \| tail -n +2 \| wc -l` = 270,761; withdrawn list in `docs/corpus.md` |
| x86 in-source: **46** crates, **128** call sites | `crates.csv` / `callsites.csv` | `SELECT count(*) FROM 'crates.csv' WHERE num_callsites>0;` and `SELECT sum(num_callsites) FROM 'crates.csv';` |
| x86 category split 23 / 10 / 4 / 9 (uncond/compile/runtime/oos) | `crates.csv` | `SELECT category, count(*) FROM 'crates.csv' WHERE num_callsites>0 GROUP BY category;` |
| AArch64: **8** crates, **14** sites, split 5 / 3 / 0 / 0 | `arm_crates.csv` / `arm_callsites.csv` | same queries on the `arm_` files |
| Build-config exposed: **382** (x86) / **381** (ARM) | `crates.csv` / `arm_crates.csv` | `SELECT count(*) FROM 'crates.csv' WHERE build_config_exposure;` |
| Compile-time closure: **39,075** crates = **13.20%**; 1.80% downloads; getrandom 2,622 direct dependents | `closure.csv` + `closure_summary.md` | `SELECT count(*) FROM 'closure.csv' WHERE seed_class IN ('compile_time','both');`; shares in `closure_summary.md` |
| Include-optional upper bound: **210,073** = 70.98% | `closure_maxbound.csv` + `closure_summary_maxbound.md` | `SELECT count(*) FROM 'closure_maxbound.csv' WHERE seed_class IN ('compile_time','both');` (wrapper-only rows excluded, as in the summary); share in the summary |
| Binary verify (x86): getrandom SGX object = **120 RDRAND, 0 CPUID** | `verify_results.csv` | `SELECT rdrand, cpuid FROM 'verify_results.csv' WHERE name='getrandom' AND config='sgx';` |
| Binary verify (ARM): getrandom = **15 `mrs RNDR`**, no feature branch | `arm_verify_results.csv` | `SELECT rndr FROM 'arm_verify_results.csv' WHERE name='getrandom' AND config='backend';` |
| Recall: 46 in-source; **178-crate** candidate pool; source-visible miss set empty (2 excluded intrinsic-defs) | `expand_validation.md` + `expand_*.csv` | narrated in `expand_validation.md`; pool = `expand_candidates.csv` |
| Precision: interprocedural over-count in **3 of 46** x86 crates | `audit_results.md` | narrated in `audit_results.md` / `audit_worksheet.md` |

Example (reproduces the paper's x86 prevalence table, Listing 1):
```sql
-- from results/
SELECT category, count(*) AS crates
FROM   'crates.csv'
WHERE  num_callsites > 0
GROUP  BY category
ORDER  BY crates DESC;
```

Full column descriptions: `docs/SCHEMA.md`. Corpus construction and exclusions:
`docs/corpus.md`.

---

## 6. Determinism notes & known limits

- **Classifier / revdep are deterministic in content, not necessarily row
  order.** Thread count (`--threads`) doesn't affect the *set* of rows or
  their values, but `classifier`'s parallel (rayon) writer means
  `crates.csv`/`callsites.csv` row order can vary run to run — sort before
  diffing (see `docs/VERIFY_FROM_ARCHIVE.md`). `revdep`'s summary `.md`
  files inherit crate names from that same row order for their "Seeds
  (N): ..." lines, so `revdep` explicitly sorts those before printing —
  the summary text itself is fully deterministic even though its input
  wasn't.
- **Binary verification** is pinned to `rustc 1.95.0`. A different compiler
  version can legitimately change emitted opcode counts; do not compare across
  toolchains.
- **Recall "empty miss set"** is established *for this pinned snapshot* (see
  `expand_validation.md`) — it is not a generalization guarantee. The
  classifier's recall gaps were repaired on the earlier 2026-05-13 snapshot;
  re-running the exhaustive scan on this later, larger snapshot re-confirms an
  empty source-visible miss set (so the held-out-snapshot recall run that was
  once future work is now done at least once).
- The **10 withdrawn crates** (`docs/corpus.md`) return HTTP 403 permanently and
  will never re-fetch; they are a fixed 0.0037% exclusion, not a transient
  error.
