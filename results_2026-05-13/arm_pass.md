# ARM RNDR/RNDRRS Pass (PLAN.md section 2/3 cross-ISA extension)

Dated 2026-07-07. Extends the classifier to AArch64 `FEAT_RNG` hardware
entropy (`RNDR`, `RNDRRS`; ARMv8.5) and runs it on the same pinned
corpus (crates.io db-dump 2026-05-13, 252,202 non-yanked crate-versions,
`snapshot.toml`). Broadens the study from "x86 hardware entropy" to
"hardware-entropy instructions across ISAs" and gives an x86-vs-ARM
prevalence comparison.

## Detection targets

`RNDR`/`RNDRRS` have **no standalone instruction**: both are read with
`MRS <Xt>, RNDR|RNDRRS` from system registers. Verified empirically on
an aarch64 host (rustc 1.95.0, `llvm-objdump`): both the named form
`mrs x, rndr` and the raw-encoding form `mrs x, s3_3_c2_c4_0/_1` assemble
to the same instruction (`d5 3b 24 0N` / `d5 3b 24 2N`) and disassemble
back to the `RNDR`/`RNDRRS` sysreg names. So the classifier detects:

- **Intrinsics** `core::arch::aarch64::__rndr` / `__rndrrs`. As of stable
  1.95 these are still behind an unstable `stdarch` gate (a plain
  `use std::arch::aarch64::__rndr` fails to resolve on stable), so real
  emitters overwhelmingly use inline `asm!`. Names are still detected for
  nightly-built crates.
- **Inline asm** `rndr` / `rndrrs` (named sysreg) and `s3_3_c2_c4_0` /
  `s3_3_c2_c4_1` (raw encoding) inside `mrs`. `contains_word` boundaries
  keep `rndr` from matching inside `rndrrs`.
- **Runtime gate** `is_aarch64_feature_detected!("rand")`.
- **Compile-time gate** `#[target_feature(enable = "rand")]`,
  `#[cfg(target_feature = "rand")]`, or an SGX target gate.

Implementation: findings are tagged with an `Isa` (`ast.rs`) and split by
ISA in one corpus walk, so the committed x86 dataset is unchanged
(`crates.csv` / `callsites.csv` are content-identical modulo the
parallel-writer row order) and ARM lands in `arm_crates.csv` /
`arm_callsites.csv` with the same schema.

## Headline comparison

| | x86 RDRAND/RDSEED | ARM RNDR/RNDRRS |
|---|---:|---:|
| in-source crates | **40** | **5** |
| call sites | 119 | 10 |
| `compile_time` | 7 | 1 |
| `unconditional` | 21 | 4 |
| `runtime` | 3 | 0 |
| `out_of_scope` | 9 | 0 |

ARM hardware-entropy use is **~8x rarer** than x86 in the same corpus --
itself a reportable result. `RNDR` is newer (ARMv8.5, 2018) and ARM-only,
so a much smaller share of the ecosystem uses it, and no ARM emitter in
the corpus sits in an SGX/kernel/TEE context (the x86 `out_of_scope`
bucket has no ARM analogue here).

## The five ARM crates

| Crate | Version | Sites | Category | Signal |
|---|---|---:|---|---|
| `getrandom` | 0.4.2 | 1 | **compile_time** | `#[target_feature(enable="rand")]` on the `rndr()` asm fn |
| `aarch64-cpu` | 11.2.0 | 2 | unconditional | `mrs rndr`/`rndrrs`, detection in a separate `new()` |
| `cortex-a` | 8.1.1 | 2 | unconditional | same shape as `aarch64-cpu` (shared lineage) |
| `armv8` | 0.0.1 | 2 | unconditional | `mrs rndr` / `rndrrs` register wrappers |
| `lcpfs` | 2026.1.102 | 3 | unconditional | `mrs rndr`/`rndrrs` in `src/arch/aarch64.rs` |

The structure mirrors the x86 findings:

- **`getrandom` is compile_time on both ISAs.** Its `rndr` backend wraps
  the `mrs` in `#[target_feature(enable = "rand")]` and uses
  `#[cfg(target_feature = "rand")]` -> always-available vs
  `is_aarch64_feature_detected!("rand")` -- the exact compile-time /
  runtime split as its x86 backend. This is the ARM analogue of the
  dissertation's headline compile-time-elision case.
- **The register-wrapper crates** (`aarch64-cpu`, `cortex-a`, `armv8`)
  gate the `mrs` only on `#[cfg(target_arch = "aarch64")]` with the
  feature discovery in a *different* function (a `new()`/constructor), so
  the function-local rule labels the call site `unconditional`. This is
  the same interprocedural undercount documented for the x86 wrappers
  (`rdrand`, `x86`, `x64`; see `second_pass.md`), now present on ARM too.
- **`getrandom` and `lcpfs` appear on both ISA lists** -- portable crates
  that carry an x86 and an AArch64 backend.

## Transitive exposure (reverse-dependency closure)

Seeded from the ARM `compile_time` crate (`getrandom`) plus the AArch64
register wrappers (`aarch64-cpu`, `cortex-a`, `armv8`); same edge model as
the x86 closure (`closure_summary*.md`). Full tables in
`arm_closure_summary*.md`.

| ARM closure | lower bound | upper bound |
|---|---:|---:|
| compile-time (getrandom) crates | 34,000 (12.78%) | 186,921 (70.2%) |
| compile-time downloads-weighted | 1.84% | -- |
| register-wrapper crates | 142 (0.05%) | -- |

The ARM compile-time closure is essentially the x86 one (34,000 vs
34,014): both are driven by `getrandom`, which is a foundational
dependency regardless of ISA. The x86 side had six extra tiny
`compile_time` seeds (`rmath`, `typed_arch`, ...); ARM has only
`getrandom`, hence the 14-crate gap. So a project that transitively pulls
`getrandom` ships *both* the x86 (SGX/`+rdrand`) and the AArch64
(`+rand`) compile-time entropy path -- the closure is one population, two
instructions.

## Precision spot-check

Hand-verified against source (analogue of the x86 audit):

- `getrandom` `src/backends/rndr.rs`: `#[target_feature(enable="rand")]`
  wraps `asm!("mrs {} , rndr", ...)` -> correctly `compile_time`.
- `aarch64-cpu` `src/asm/random.rs`: `mrs rndr` in `Rndr::rndr()` gated by
  `#[cfg(target_arch="aarch64")]` only; discovery is in `Rndr::new()` ->
  correctly `unconditional` under the function-local rule.

Unit tests (`ast.rs` `mod tests`) cover: ARM intrinsic + runtime gate,
named-sysreg asm + `target_feature="rand"`, raw-encoding `s3_3_c2_c4_1`
== RNDRRS, `rndr`/`rndrrs` word-boundary separation, and x86 unchanged.

## Interception contrast (discussion, not a measurement)

`RNDR`/`RNDRRS` are MRS reads of system registers and appear to be
*directly* trappable (`FEAT_RNG_TRAP` / an `SCR_EL3.TRNDR`-style control),
unlike x86 `RDRAND`, which is interceptable only via the indirect
CPUID-faulting mechanism this work targets. If so, ARM sidesteps the
compile-time-gating gap entirely -- a point for the discussion chapter.
Per the project's hardware-claim rule, the trap mechanism must be
verified against the ARM Architecture Reference Manual before it is
asserted as fact; this pass establishes only the *instruction form* (MRS
sysreg read), which is verified.

## Build-config exposure

Now scanned for ARM (see `buildconfig_closure.md`): the build-config pass
detects `-C target-feature=+rand` and treats `target-cpu=native` as
ISA-neutral. Result: **388 ARM build-config-exposed crates**, essentially
all via `target-cpu=native` (zero explicit `+rand`) -- ARM entropy
build-config exposure is inherited through `native`, shared with x86, not
hand-set. Their reverse-dependency closure is tiny (1,054 crates, 0.40%,
0.01% downloads), i.e. these are leaf applications, in stark contrast to
the 12.78% compile-time-source closure above.

## Limitations

- **Unstable intrinsics.** `__rndr`/`__rndrrs` are nightly-gated, so
  intrinsic-based emitters are rare today; the population may grow when
  they stabilize. Detection already covers the names.
- Same monomorphization / interprocedural caveats as the x86 pass.

## Reproducing

```
cd rust-study
pipeline/classifier/target/release/classifier --out results/
# -> results/{crates,callsites}.csv (x86, unchanged)
#    results/{arm_crates,arm_callsites}.csv (AArch64)
```
