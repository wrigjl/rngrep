# Reproducing rngrep

This document tells a researcher how to reproduce **every number** in the paper
*"Mining RDRAND: A Dataset of Hardware-Entropy Instruction Use Across the Rust
Crate Ecosystem"* from the pinned crates.io snapshot, and — for each number —
which file and command produces it.

Read it top to bottom once before starting: the corpus fetch is large (~34 GB
of tarballs, ~150 GB decompressed working set) and the db-dump pin has a
prerequisite (§2) that you must satisfy *before* the closure numbers can be
reproduced.

---

## 1. What is reproducible, and how

The pipeline has two independent inputs, with different reproduction stories:

| Input | Pins it | Reproducible? |
|---|---|---|
| The `.crate` source corpus (252,202 tarballs) | `results/manifest.csv.gz` (crate→version→sha256) + `snapshot.toml` | **Byte-for-byte.** Each tarball is re-fetchable from `static.crates.io` and verified against its recorded SHA-256. |
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
sha256 = db6bf2ac1b1217d627bcae1594d0f78e32ded593eb128a6fbf174efb2e71e974
fetched_at = 2026-05-13T17:40:08Z      # dump directory: 2026-05-13-020017
```

That URL always serves **the current day's** dump. crates.io does **not**
publish historical dumps at a retrievable URL. So a fresh `crawler db-dump`
today downloads a *different* population and every closure number will differ.

To reproduce faithfully, obtain the archived pinned dump and stage it:

The pinned `db-dump.tar.gz` (1.15 GB) is archived in two places:

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
> `db-dump.tar.gz` (1.15 GB) in addition to `manifest.csv.gz`, or the closure
> numbers are not reproducible by a third party. Upload it to Zenodo as an
> explicit file: the GitHub→Zenodo integration snapshots only the repo tarball
> and captures neither release assets nor LFS objects. The classifier corpus
> does not need archiving — the manifest reconstructs it from
> `static.crates.io`. The dump is deliberately **not** committed to git (1.15 GB
> blows GitHub's 100 MB file limit, and LFS its free quota).

---

## 3. Prerequisites

| Need | For | Install |
|---|---|---|
| `rustup` + rustc **1.95.0** | everything | `rust-toolchain.toml` auto-selects 1.95.0 in this directory; rustup installs it (and the `x86_64-unknown-linux-gnu` + `x86_64-fortanix-unknown-sgx` targets listed there) on first `cargo` invocation. |
| ~150 GB scratch + network | corpus | disk for tarballs + decompressed working set; access to `static.crates.io`. |
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
crawler fetch                                  # ~34 GB of .crate tarballs
```
`fetch` is idempotent/append-only: it verifies each tarball against the
`checksum` column of the dump's `versions.csv` and records
`name,version,sha256,size_bytes,fetched_at` in `manifest.csv`. Confirm your
`manifest.csv` has **252,202** data rows and matches the shipped
`results/manifest.csv.gz`.

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
narrated in `verify_summary.md` / `arm_verify_summary.md`.

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
| 266,128 crates in db-dump; 301,453,258,010 total downloads | db-dump | header of `results/closure_summary.md` (from Stage 3) |
| 252,211 target crates; **252,202** fetched; 9 unrecoverable (403) | corpus | `zcat results/manifest.csv.gz \| tail -n +2 \| wc -l` = 252,202; withdrawn list in `docs/corpus.md` |
| x86 in-source: **40** crates, **119** call sites | `crates.csv` / `callsites.csv` | `SELECT count(*) FROM 'crates.csv' WHERE num_callsites>0;` and `SELECT sum(num_callsites) FROM 'crates.csv';` |
| x86 category split 21 / 7 / 3 / 9 (uncond/compile/runtime/oos) | `crates.csv` | `SELECT category, count(*) FROM 'crates.csv' WHERE num_callsites>0 GROUP BY category;` |
| AArch64: **5** crates, **10** sites, split 4 / 1 / 0 / 0 | `arm_crates.csv` / `arm_callsites.csv` | same queries on the `arm_` files |
| Build-config exposed: **389** (x86) / **388** (ARM) | `crates.csv` / `arm_crates.csv` | `SELECT count(*) FROM 'crates.csv' WHERE build_config_exposure;` |
| Compile-time closure: **34,014** crates = **12.78%**; 1.84% downloads; getrandom 2,117 direct dependents | `closure.csv` + `closure_summary.md` | `SELECT count(*) FROM 'closure.csv' WHERE seed_class IN ('compile_time','both');`; shares in `closure_summary.md` |
| Include-optional upper bound: **186,906** = 70.2% | `closure_maxbound.csv` + `closure_summary_maxbound.md` | `SELECT count(*) FROM 'closure_maxbound.csv' WHERE seed_class IN ('compile_time','both');` (the 41 wrapper-only rows are excluded, as in the summary); share in the summary |
| Binary verify (x86): getrandom SGX object = **120 RDRAND, 0 CPUID** | `verify_results.csv` | `SELECT rdrand, cpuid FROM 'verify_results.csv' WHERE name='getrandom' AND config='sgx';` |
| Binary verify (ARM): getrandom = **15 `mrs RNDR`**, no feature branch | `arm_verify_results.csv` | `SELECT rndr FROM 'arm_verify_results.csv' WHERE name='getrandom' AND config='backend';` |
| Recall: population **31 → 40**; 186-crate candidate pool; 0 macro-hidden emitters | `expand_validation.md` + `expand_*.csv` | narrated in `expand_validation.md`; pool = `expand_candidates.csv` |
| Precision: interprocedural over-count in **3 of 40** x86 crates | `audit_results.md` | narrated in `audit_results.md` / `audit_worksheet.md` |

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

- **Classifier / revdep** are deterministic; thread count (`--threads`) affects
  only speed, not output.
- **Binary verification** is pinned to `rustc 1.95.0`. A different compiler
  version can legitimately change emitted opcode counts; do not compare across
  toolchains.
- **Recall "empty miss set"** is established *for this pinned snapshot after
  iterative repair* (see `expand_validation.md`) — it is not a generalization
  guarantee. A held-out-snapshot recall run is future work.
- The **9 withdrawn crates** (`docs/corpus.md`) return HTTP 403 permanently and
  will never re-fetch; they are a fixed 0.0036% exclusion, not a transient
  error.
