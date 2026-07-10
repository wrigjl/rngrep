# Macro-expansion / false-negative validation (2026-07-08 rerun)

Recall re-validation of the static classifier against the **2026-07-08**
snapshot (`af2a04c11d...`), 270,761 crate-versions, license-filtered
corpus. Parallels `../results/expand_validation.md` (the 2026-05-13
pass); read that first for the method rationale. Run 2026-07-09 on
`aarch64-apple-darwin` against the pinned corpus in `~/scratch2`.

## Why not a random sample

The classifier flags 46 / 270,761 crates (rate 1.7e-4). As on the May
snapshot, a random sample cannot bound a defect this rare, so we
**enumerate candidates exhaustively** and rule out macro generation with
a pre-expansion source scan.

## Pipeline (checked-in tools, unchanged from the May pass)

1. `pipeline/audit/corpus_grep.py` (CRATES_SCRATCH_DIR=~/scratch2):
   **301** crates mention `rdrand`/`rdseed` in `.rs` (loose substring;
   0 tarball errors). May: 293.
2. `pipeline/audit/corpus_grep_refine.py` (word-boundary): **225**
   strict hits (dropped 76 substring-noise). Sanity: all **46** known
   in-source crates still strict-match.
3. Minus classifier-known -> **178-crate candidate pool**
   (`expand_candidates.csv`). May: 186.
4. `pipeline/audit/candidate_source_scan.py` (authoritative
   pre-expansion source scan over all 178) -> `expand_missed.csv`.

Note: the `cargo expand` ladder (`expand_run.sh`, kiriko/x86 nightly)
was **not** re-run for this snapshot. Its sole distinct contribution on
the May pass was the negative "no macro-*generated* emitter" -- a
structural property of the ecosystem (Rust crates do not token-paste
these intrinsics), established there and not a moving target. The
pre-expansion source scan is what makes the recall claim sound (per the
May write-up), and it is re-run here.

## Headline

- **0 genuine downstream misses.** The source scan surfaces only
  **2** crates, both intrinsic *definitions* (`core_arch` 0.1.5,
  `coresimd` 0.1.2 -- published copies of stdarch `core::arch`),
  excluded by the same rule as the `rdrand`/`x86` wrapper libraries, not
  downstream call sites. `expand_missed.csv` lists them.
- **The source-visible miss set is empty for the July snapshot**
  (modulo deliberately obscured/computed encodings), exactly as for May.
- **2 FFI-extern-only** (`rust-halcon`, `spout_texture_share`) carry
  bindgen extern decls, no rustc-emitted RDRAND -- excluded, same as May.

## Why the miss set is empty now (vs. 8 genuine misses in May)

Every genuine downstream miss the May pass found (`bitcoin-random`,
`cocoon-tpm-crypto`, `lcpfs`, `virtfw-libhw`, `mbedtls`, `mbedtls-smx`,
`nanorand`, `hyper-thread-random`) was fixed in
`pipeline/classifier/src/ast.rs` (5 root-cause classes: inline-asm in
statement position, grouped/local `use ... as` aliases, intrinsic as a
macro argument, `.byte` opcode encoding). Those crates are now
classifier-known, so they drop out of the candidate pool -- the empty
miss set is the fixed classifier applied to a fresh, larger corpus, not
a re-derivation of the same result.

## Artifacts

- `expand_grep_raw.csv` -- 301 loose mnemonic hits.
- `expand_candidates.csv` -- the 178-crate strict candidate pool.
- `expand_missed.csv` -- the 2 intrinsic-definition "misses" (excluded).
