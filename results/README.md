# First-pass classifier results

Snapshot: crates.io db-dump dated 2026-05-13, corpus of **252,202**
non-yanked, non-prerelease crate-versions (see `../corpus.md`).

Output files in this directory are produced by
`pipeline/classifier/target/release/classifier --out results/`.

## Headline numbers

Crates with at least one RDRAND/RDSEED call site:

| Category | Crates | Description |
|---|---:|---|
| `unconditional` | 12 | RDRAND emitted with no in-crate gate; caller responsibility. The rr-relevant unsafe population. |
| `compile_time` | 6 | Gated by `#[target_feature(enable = "rdrand"\|"rdseed")]`, `#[cfg(target_feature = "rdrand"\|"rdseed")]`, or `#[cfg(target_env = "sgx")]` / `target_os = "sgx"`. CPUID elided when the gate is true. |
| `runtime` | 3 | Function body contains `is_x86_feature_detected!("rdrand"\|"rdseed")` — rr can intercept. |
| `out_of_scope` | 9 | Crate runs in a context rr cannot record (SGX enclave, unikernel/OS kernel, or non-SGX TEE). `out_of_scope_reason` records which (`sgx` / `kernel` / `tee`). |
| **Total with RDRAND/RDSEED in source** | **30** | Across 80 distinct call sites |

Source-level signals take precedence over out-of-scope detection — a
crate with both `#[target_feature]` source patterns and an SGX
marker stays `compile_time`.

Crates with build-config exposure but no in-source RDRAND code:

| Exposure | Crates |
|---|---:|
| `target-cpu=native` (implicit, host-dependent) | 387 |
| `-C target-feature=+rdrand` (direct) | 1 |
| Other (custom `cargo:rustc-cfg=` patterns, etc.) | 23 |

## What "compile_time" identifies

The proposal's [problem.tex:173-207](../../problem.tex#L173-L207) calls out
the compile-time RDRAND pattern as the case where `rr`'s CPUID faulting
is irrelevant. The first-pass classifier flags six crates as direct
matches:

| Crate | Version | Pattern |
|---|---|---|
| `getrandom` | 0.4.2 | `#[target_feature(enable = "rdrand")] fn rdrand()` |
| `safe_arch` | 1.0.0 | Wrapper crate, `#[target_feature]`-attributed |
| `typed_arch` | 0.1.1 | Wrapper crate, same pattern |
| `concrete-csprng` | 0.4.1 | CSPRNG with target-feature gating |
| `tfhe-csprng` | 0.9.0 | TFHE CSPRNG, same pattern |
| `rmath` | 0.1.5 | Custom `rustc-cfg=rmath_rdrand=` in `build.rs` |

A separate cluster of nine crates is detected by `sgx.rs`,
`kernel.rs`, and `tee.rs` and reported as `out_of_scope`. These run
in SGX enclaves, KVM guest kernels, unikernels (HermitOS), no-std OS
kernels (Asterinas), or non-SGX TEEs — none of which rr can record.
Per-reason breakdown: 5 SGX, 3 kernel, 1 TEE.

## Known undercounts (first pass)

The classifier flags only patterns visible from a crate's own source.
The following populations are systematically under-reported and require
follow-up:

1. **Mixed-use library wrappers.** `rdrand`, `x86`, `x86_64`, `x64`,
   `lokacore`, `mc-rand` are libraries that emit RDRAND without
   in-crate gating; whether the resulting binary is in scope for rr
   depends on the downstream consumer (kernel-mode vs. userspace).
   The classifier counts them as `unconditional`; the dissertation
   should report downloads-weighted / rdep-weighted shares (`next.md`
   step 3) so the headline reflects what rr would actually encounter.

2. **Distant runtime checks.** Crates like `rdrand`, `x86`, `x86_64`
   structure CPUID checks in a `new()` constructor while the actual
   RDRAND call lives in a `next_u32()`-style method. The classifier's
   function-local heuristic doesn't follow this dataflow and labels
   them `unconditional`. Interprocedural analysis or hand-classification
   of these ~5 crates is appropriate.

3. **`#[cfg(not(target_feature = "rdrand"))]` CPUID branches.** The
   `getrandom` pattern is to gate the CPUID check itself with
   `#[cfg(not(target_feature = "rdrand"))]`, so that when `+rdrand` is
   set the check is omitted. The classifier currently catches the
   `#[target_feature(enable = ...)]` half of this pattern but not the
   `#[cfg(not(...))]` half independently.

4. **Macro-generated RDRAND.** Beyond `cfg_if!` (handled by the
   classifier's macro-body token scan), pre-expansion source misses
   intrinsics produced by `paste!`, custom proc-macros, and similar.
   The PLAN's `cargo expand` sub-sample is the documented mitigation.

## Reproduction

```
cd rust-study
pipeline/classifier/target/release/classifier --out results/
```

Runtime: ~100 s on the 252k-crate corpus on an Apple Silicon laptop
(rayon parallelism over logical cores).

## Output schema

`callsites.csv` — one row per detected RDRAND/RDSEED call site.
Columns: `name, version, file, line, kind, detail, cfg_gates,
in_test_or_bench, runtime_gated, compile_gate_present,
target_feature_attr, category`.

`crates.csv` — one row per crate that either has at least one call
site or has build-config exposure. Columns: `name, version,
num_callsites, num_intrinsic, num_inline_asm, n_runtime,
n_compile_gate, n_unconditional, n_in_test, category,
build_config_exposure, build_config_detail, parse_failures`.

`category` at the crate level is the worst (highest-severity) non-test
category across all call sites: `unconditional > compile_time > runtime
> none`.
