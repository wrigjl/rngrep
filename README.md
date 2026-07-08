# rngrep

**Grep the Rust ecosystem for hardware-entropy instructions.**

`rngrep` statically analyses every non-yanked crate on crates.io for use of
the hardware random-number instructions — x86 **RDRAND/RDSEED** and AArch64
**RNDR/RNDRRS** — and classifies each call site by how (or whether) its
feature-detection gate is applied. It then computes the reverse-dependency
closure of the affected population and verifies a sample against emitted
machine code.

This repository is the reproducible artifact for the paper *"Mining RDRAND: A
Dataset of Hardware-Entropy Instruction Use Across the Rust Crate Ecosystem"*
(MSR Data & Tool Showcase). It contains the pipeline, the derived dataset, and
a corpus manifest pinned to a crates.io snapshot.

## Why

Deterministic record-and-replay tools (e.g. `rr`) intercept hardware entropy
by faulting `CPUID`. That fails when a binary reads `RDRAND` **without** a
CPUID gate — the compile-time-gated pattern. `rngrep` measures how widespread
that pattern is across the ecosystem, on two ISAs.

## Headline numbers

Corpus: crates.io db-dump **2026-05-13**, **252,202** non-yanked,
non-prerelease crate-versions (`snapshot.toml`).

| | x86 RDRAND/RDSEED | AArch64 RNDR/RNDRRS |
|---|---:|---:|
| in-source crates | 40 | 5 |
| call sites | 119 | 10 |
| compile-time gated | 7 | 1 |
| unconditional | 21 | 4 |
| runtime gated | 3 | 0 |
| out-of-scope (SGX/kernel/TEE) | 9 | 0 |
| build-config exposed | 389 | 388 |
| compile-time reverse-dep closure | 12.78% of crates (1.84% downloads) | 12.78% |

Binary verification confirms the headline: `getrandom`'s compile-time backend
emits a bare `RDRAND` / `mrs RNDR` retry loop with **no** feature-detection in
the compiled path — the pattern `rr`'s CPUID faulting cannot intercept.

## Layout

```
pipeline/        the tool
  crawler/       fetch the crates.io db-dump + .crate sources
  classifier/    static AST/asm analysis; ISA-tagged call-site classification
  revdep/        reverse-dependency closure (compile-time & build-config seeds)
  audit/         hand-audit + macro-expansion (recall) validation scripts
  verify/        binary verification (build 3 configs, disassemble, grep opcode)
results/         the dataset (see docs/SCHEMA.md)
  crates.csv, callsites.csv              x86 per-crate / per-call-site
  arm_crates.csv, arm_callsites.csv      AArch64
  closure*.csv, arm_closure*, *bc_*      reverse-dependency closures
  verify_results.csv, arm_verify_*       binary-verification outcomes
  *_summary.md, arm_pass.md, ...         result writeups
  manifest.csv.gz                        crate -> version -> sha256 (corpus pin)
docs/            SCHEMA.md, corpus.md (corpus construction)
snapshot.toml    pinned db-dump URL + sha256
rust-toolchain.toml  pinned rustc (1.95.0)
reproduce.sh     end-to-end driver
```

## Reproduce

Requires the pinned toolchain (`rust-toolchain.toml` selects rustc 1.95.0) and
~150 GB of scratch for the corpus working set. See `reproduce.sh` for the full
sequence; in brief:

```bash
# 1. corpus (pinned by snapshot.toml; ~34 GB of .crate tarballs)
( cd pipeline/crawler && cargo run --release -- db-dump )
( cd pipeline/crawler && cargo run --release -- fetch )

# 2. classify the whole corpus -> results/{crates,callsites}.csv + arm_*
pipeline/classifier/target/release/classifier --out results/

# 3. reverse-dependency closures
pipeline/revdep/target/release/revdep                       # x86 compile-time (lower)
pipeline/revdep/target/release/revdep --include-optional    # upper bound
pipeline/revdep/target/release/revdep --crates-csv results/arm_crates.csv \
    --wrappers "aarch64-cpu,cortex-a,armv8" --label arm_    # ARM
pipeline/revdep/target/release/revdep --seed-mode build_config \
    --wrappers "" --label bc_                               # build-config
```

The corpus itself is **not** stored here (re-fetchable from
`static.crates.io`); `results/manifest.csv.gz` pins the exact
crate→version→sha256 set so it can be reconstructed byte-for-byte. Binary
verification (`pipeline/verify/`) runs on a native host per ISA (x86_64 for
RDRAND, aarch64 for RNDR); see `results/verify_summary.md` and
`results/arm_verify_summary.md`.

## Data & schema

Column-by-column descriptions of every CSV are in `docs/SCHEMA.md`; corpus
construction and threats to validity are in `docs/corpus.md`. The result
writeups (`results/*_summary.md`, `results/arm_pass.md`,
`results/buildconfig_closure.md`, `results/verify_summary.md`) narrate each
analysis.

## Citation & license

Cite via `CITATION.cff` (and the paper once published). Code is MIT
(`LICENSE`); the derived dataset is CC BY 4.0 (`LICENSE-DATA`).
