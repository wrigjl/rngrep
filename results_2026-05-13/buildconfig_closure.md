# Build-Config Exposure and Closure (x86 + ARM)

Dated 2026-07-07. Completes the rubric's 4th bucket (PLAN.md section 2/4):
crates whose *packaging* force-enables a hardware-entropy instruction via
rustflags, propagating it to the crate's build. Adds the AArch64 `+rand`
scan and computes the reverse-dependency closure of the build-config
population for both ISAs, alongside a correctness fix to the detector.

## What build-config exposure is

A crate can turn the compile-time gate true for its whole build tree by
setting, in `Cargo.toml`, `.cargo/config.toml`, or `build.rs`:

- `-C target-feature=+rdrand` / `+rdseed` (x86, explicit),
- `-C target-feature=+rand` (AArch64 FEAT_RNG, explicit),
- `-C target-cpu=native` (ISA-neutral: enables whatever the build CPU
  has -- `+rdrand` on x86, `+rand` on an AArch64 host with FEAT_RNG).

This is what flips `getrandom`'s `#[cfg(target_feature = "rand")]` /
`#[cfg(target_feature = "rdrand")]` path from runtime-detected to
unconditional (confirmed in `verify_summary.md` / `arm_verify_summary.md`).

## Detector fix: bogus `native` build-config rows removed

Extending the scan to `+rand` surfaced a pre-existing false-positive class
in the x86 data. `buildconfig::scan_build_rs` matched the bare substring
`native`, which over-fired on `build.rs` lines that are **not**
`target-cpu=native`:

- linker search paths: `cargo:rustc-flags=-L native={}` (the `native`
  link-kind), and
- custom cfg names: `cargo:rustc-cfg=native`, `rmath_rdrand`,
  `lilos_has_native_rmw`, `sp1_native_executor`, ...

Exposure is now gated by `BuildConfigFinding::exposes_x86/exposes_arm`,
which require the full `target-cpu=native` (or `+rdrand`/`+rand`) token.
This drops **22 spurious x86 build-config rows** (411 -> **389**) and
flips one in-source crate's flag (`rmath`, whose `build.rs` sets a custom
`rmath_rdrand` cfg, not a rustflag). `rmath`'s *category* stays
`compile_time` (it comes from call sites), so the compile-time closure and
all binary-verification results are unaffected; `callsites.csv` is
unchanged. Net: the x86 dataset is strictly more correct.

## Build-config population

| | x86 | ARM |
|---|---:|---:|
| build-config-exposed crates | **389** | **388** |
| via `target-cpu=native` | 388 | 388 |
| explicit `+rdrand` / `+rand` | 1 | 0 |

The ARM and x86 populations are almost the same crates: exposure is
overwhelmingly `target-cpu=native`, which is ISA-neutral, so the same
crates expose RDRAND (built on x86) and RNDR (built on AArch64). Explicit
hand-set entropy features are near-nil on both ISAs (1 x86, 0 ARM) --
itself a reportable result.

## Build-config closure (reverse-dependency)

Reverse-dependency closure seeded from the build-config-exposed crates
(no wrapper seeds), same edge model as the compile-time closure. Files:
`bc_closure_summary.md` (x86), `arm_bc_closure_summary.md` (ARM).

| Reverse closure of build-config seeds | x86 | ARM |
|---|---:|---:|
| crates in closure | 1,056 (0.40%) | 1,054 (0.40%) |
| direct dependents (depth 1) | 455 | 454 |
| downloads-weighted share | 0.01% | 0.01% |

### Interpretation -- leaves vs. foundations

This is the headline contrast of the build-config analysis:

- **Compile-time *sources* sit at the foundations.** `getrandom` is a
  near-universal dependency, so its reverse closure is **12.78%** of the
  ecosystem (1.84% downloads-weighted). Danger propagates *up* the
  dependency graph: anything that pulls `getrandom` ships its
  compile-time source.
- **Build-config *setters* sit at the leaves.** The 389/388 crates that
  set an entropy build-flag have a reverse closure of only **0.40%** and a
  **0.01%** download share -- they are overwhelmingly leaf applications and
  binaries with few or no dependents. Build-config exposure does **not**
  amplify up the graph.

Direction caveat: rustflags in a crate's `.cargo/config.toml` /
`Cargo.toml` apply to that crate's own build tree (its *dependencies*,
forward) only when it is the build root; they are **not** inherited by its
dependents. So the reverse closure above is the correct way to show
build-config setters are non-foundational; the *forward* closure (the
union of the setters' dependency trees -- the libraries actually compiled
with the flag) is the complementary exposure surface and is left as future
work. Either way the conclusion holds: build-config exposure is a
per-application, opt-in choice, not an ecosystem-propagating hazard like a
compile-time source in a foundational crate.

## Reproducing

```
cd rust-study
pipeline/classifier/target/release/classifier --out results/   # +rand scan, FP fix
# build-config closures (no wrapper seeds):
pipeline/revdep/target/release/revdep --seed-mode build_config --wrappers "" --label bc_
pipeline/revdep/target/release/revdep --crates-csv results/arm_crates.csv \
    --seed-mode build_config --wrappers "" --label arm_bc_
# ARM transitive (compile-time) closure:
pipeline/revdep/target/release/revdep --crates-csv results/arm_crates.csv \
    --wrappers "aarch64-cpu,cortex-a,armv8" --label arm_
pipeline/revdep/target/release/revdep --crates-csv results/arm_crates.csv \
    --wrappers "aarch64-cpu,cortex-a,armv8" --label arm_ --include-optional
```
