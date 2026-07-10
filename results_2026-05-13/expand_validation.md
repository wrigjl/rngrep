# Macro-expansion / false-negative validation

Validation of the static classifier's **recall**: does it miss RDRAND/RDSEED
emission that is hidden from a pre-expansion `syn` AST -- macro-generated
intrinsics, or other forms the detectors don't reach? PLAN section 9 /
`next.md` step 4. Run 2026-07-06/07, corpus pinned to the 2026-05-13
snapshot (`snapshot.toml`).

## Why not a random sample

The classifier flags 31 / 252,202 crates (rate 1.2e-4). A random sample
of ~45 crates -- the PLAN's original sketch -- expects
`45 * 1.2e-4 ~= 0.005` hits: it can neither find a rare missed emitter nor
bound the false-negative rate to anything tighter than a few percent
(three orders of magnitude above the base rate). Rare defects are not
estimable by small random samples.

So instead we **enumerate candidates exhaustively**: grep every crate's
`.rs` text for the mnemonics `rdrand`/`rdseed`, subtract the crates the
classifier already flagged, and treat the remainder as the complete pool
of crates that *mention* the instruction but were *not* flagged. Every
missed emitter with any textual trace is necessarily in that pool. The
`cargo expand` pass then rules out macro generation among them.

## Pipeline

1. Corpus grep (`pipeline/audit/corpus_grep.py`): 293 crates mention
   `rdrand`/`rdseed` in `.rs` (broad substring).
2. Word-boundary refine (`corpus_grep_refine.py`): 219 strict hits;
   dropped 74 substring-noise (`keywo|rdseed|`, `reco|rdrand|omizer`).
   Sanity: all 31 known in-source crates still match.
3. Minus classifier-known -> **186-crate candidate pool**
   (`results/expand_candidates.csv`, auto-bucketed).
4. `cargo expand` fallback ladder on kiriko (native x86_64, nightly):
   all-features -> targeted rdrand/rdseed features -> default
   (`pipeline/audit/expand_run.sh`). 119 expanded, 67 failed (36%).
5. Authoritative pre-expansion source scan over all 186
   (`candidate_source_scan.py`) -- needed because expansion fails for
   ~1/3 of the pool, so build success cannot gate detection.

## Headline

- **0 truly macro-hidden emitters.** No crate emits RDRAND/RDSEED only
  after macro expansion; every real emitter is visible in pre-expansion
  source text. The macro-generation worry is not borne out in the corpus.
- **10 source-visible emitters the classifier missed** (recall gaps),
  found by the exhaustive source scan (`results/expand_missed.csv`). The
  corpus-grep approach -- not random sampling -- is what surfaced these.

## The 10 missed emitters

Two are reimplementations of `core::arch` itself (they *define* the
intrinsics via LLVM-link bindings, analogous to the `rdrand`/`x86`
wrapper libraries -- a categorization judgment call, not a downstream
call site):

| Crate | Note |
|---|---|
| `core_arch` 0.1.5 | published copy of stdarch `core::arch` |
| `coresimd` 0.1.2 | older name for the same |

The other **8 are genuine downstream emitters** that should have been
flagged, in three root-cause classes:

| Crate | Emission | Root-cause class |
|---|---|---|
| `bitcoin-random` 0.1.21 | `asm!` rdrand | inline-asm missed |
| `cocoon-tpm-crypto` 0.1.3 | `asm!` rdseed | inline-asm missed |
| `lcpfs` 2026.1.102 | `asm!` rdrand/rdseed | inline-asm missed |
| `virtfw-libhw` 0.5.0 | `asm!("rdrand %rax")` | inline-asm missed |
| `mbedtls` 0.13.5 | `_rdrand32_step as _rdrand_step` | intrinsic use-alias (grouped, module-level) |
| `mbedtls-smx` 0.13.9 | same | intrinsic use-alias |
| `nanorand` 0.8.0 | `_rdseed64_step as rdseed` | intrinsic use-alias (function-local) |
| `hyper-thread-random` 0.4.2 | `generate!(u16, ...::_rdrand16_step)` | intrinsic passed as macro argument |

Excluded as non-emission: **2 FFI-extern-only** crates (`rust-halcon`,
`spout_texture_share`) carry bindgen `pub fn _rdrand16_step(...) -> ...;`
extern declarations -- the RDRAND lives in linked C, not rustc output.
The remaining ~174 candidates are disassembler tables, CPUID
feature-detection, doc comments, and downstream users of the
`rdrand`/`x86_64` crates -- correctly unflagged.

## Root-cause classes (classifier recall gaps)

1. **Inline-asm emission (4 crates).** The prefilter matches (the
   mnemonic is present) and `scan_asm_mnemonics` handles the
   word-boundary match, so the AST walk most likely never ran on the
   containing file -- a silent `syn` parse-failure skip. To be pinned
   during the fix (parse-failure vs asm-scan gap).
2. **Intrinsic `use ... as` aliases (3 crates).** The rename collector
   does not fire for grouped/module-level (`mbedtls`) or function-local
   (`nanorand`) `use ...::_rd..._step as NAME` imports; the call through
   the alias is then unrecognized.
3. **Intrinsic as a macro argument (1 crate).** `hyper-thread-random`
   passes the intrinsic path into a declarative macro; the generic
   macro-token scan misses it in this position.

## What `cargo expand` contributed

The expansion pass's distinct result is the **negative**: across the 119
crates that built, nothing emitted an intrinsic that was absent from
source -- confirming there is no macro-*generated* RDRAND to find. Note
3 of the 10 misses (`bitcoin-random`, `mbedtls`, `mbedtls-smx`) *failed*
to expand, so expansion alone would have missed them; the pre-expansion
source scan is what makes the recall claim sound. Expand coverage: 119 /
186 (64%); the 67 failures are dominated by `-sys` crates (32) needing C
libraries -- immaterial, since FFI bindings emit no rustc RDRAND.

## Impact (fixes applied -- see `third_pass.md`)

The recall gaps were fixed in `pipeline/classifier/src/ast.rs` and the
classifier re-run: **in-source population 31 -> 40** (the two stdarch
copies `core_arch`/`coresimd` stay excluded as intrinsic *definitions*,
not downstream call sites). `getrandom` was preserved as `compile_time`
after catching and fixing a rename over-fire; a before/after diff shows
no pre-existing crate changed category. The reverse-dependency closure
was recomputed (lower bound 34,007 -> 34,014; negligible).

Two refinements surfaced during the fix:

- **`bitcoin-random` is opcode-byte-encoded**, not mnemonic asm. It emits
  RDRAND as `.byte 0x0f, 0xc7, 0xf0` (the `0f c7 f?` opcode PLAN section 6
  greps in binaries), with the mnemonic only in a `//` comment. This is a
  fifth root-cause class; source detection now covers the `.byte` form.
- **The candidate pool missed `ort-openrouter-cli`** (`asm!("RDRAND rax")`).
  The pool subtracted *all* classifier-known crates, including
  build-config-only ones; `ort-openrouter-cli` had build-config exposure
  *and* an unflagged asm call site, so it was excluded from the expansion
  sample. The recall fix caught it directly on re-run (40th crate).
  Future candidate pools should subtract only crates known to have a
  *call site*, not build-config-only exposure.

Precision was established by the hand-audit (`audit_results.md`); this
pass establishes **recall**. The classifier's headline remains a lower
bound, but the exhaustive scan bounds the source-visible miss set to
empty for the pinned snapshot (modulo arbitrary computed/obfuscated
instruction encodings, a stated limitation).

## Artifacts

- `results/expand_candidates.csv` -- the 186-crate pool, bucketed.
- `results/expand_results.csv` -- per-crate expand config + detection.
- `results/expand_missed.csv` -- the 10 confirmed source-visible misses.
- `results/expand_hits/` -- expanded `.rs` for crates with any hit.
- `pipeline/audit/{corpus_grep,corpus_grep_refine,candidate_source_scan}.py`,
  `pipeline/audit/expand_run.sh` -- reproducible pipeline.
