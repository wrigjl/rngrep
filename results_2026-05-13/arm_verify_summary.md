# ARM Binary-Verification Results (binary-plan.md, cross-ISA)

Dated 2026-07-07. Confirms the ARM classifier (`arm_pass.md`) against the
machine code rustc emits, the AArch64 analogue of `verify_summary.md`.

## Setup

- **Build host:** the dev host, native `aarch64-apple-darwin` (Apple
  Silicon), so builds are native, not cross.
- **Toolchain:** stable `rustc 1.95.0` (`rust-toolchain.toml`, forced with
  `cargo +1.95.0`).
- **Corpus:** the 5 ARM in-source crates at their pinned `arm_crates.csv`
  versions, fetched from `static.crates.io`.
- **Disassembler:** `llvm-objdump -d`. RNDR/RNDRRS have no standalone
  instruction; both are `mrs <Xt>, RNDR|RNDRRS` (`d5 3b 24 0N` /
  `d5 3b 24 2N`). We count instruction lines whose mnemonic is `mrs` and
  operand is `rndr`/`rndrrs`, so symbol/path strings containing "rndr"
  (e.g. getrandom's `rndr.rs`) do not false-match.
- **Configs:** `default` (native, default features), `rand`
  (`RUSTFLAGS="-C target-feature=+rand"`), and for getrandom a `backend`
  config selecting the rndr backend
  (`--cfg getrandom_backend="rndr" -C target-feature=+rand`).

Driver: `pipeline/verify/arm_verify.sh`. Raw data:
`results/arm_verify_results.csv`.

## Headline: getrandom's rndr backend elides the runtime check

`getrandom` emits no RNDR under `default`/`+rand` (its default backend is
not rndr -- the same baseline as the x86 side). With the rndr backend
selected, `getrandom::backends::rndr::rndr` is a bare `MRS RNDR` retry
loop with **no `is_aarch64_feature_detected` in the compiled path**:

```
204 <...backends::rndr::rndr...>:
20c: d53b2401   mrs  x1, RNDR
214: b40001e8   cbz  x8, 0x250      # retry on failure (RNDR clears x8)
218: d53b2401   mrs  x1, RNDR
220: b4000188   cbz  x8, 0x250
... (15 mrs RNDR, no feature-detection branch)
```

This is the AArch64 analogue of the x86 headline: the compile-time gate
(`#[target_feature(enable = "rand")]` + `#[cfg(target_feature = "rand")]`)
removes the runtime discovery, leaving an unconditional hardware-entropy
read.

## Outcomes (5 crates)

| Crate | Category | Verdict | Emitted |
|---|---|---|---|
| `getrandom` | compile_time | **EMIT-CONFIRMED** | 15 `mrs RNDR` (backend cfg) |
| `armv8` | unconditional | **EMIT-CONFIRMED** | 1 `rndr` + 1 `rndrrs` (2+2 with +rand) |
| `lcpfs` | unconditional | **EMIT-CONFIRMED** | 93 `rndr` + 1 `rndrrs` |
| `aarch64-cpu` | unconditional | BUILT-NO-EMIT (mono) | `rndr()` un-instantiated in a bare lib |
| `cortex-a` | unconditional | BUILD-LIMITED | nightly-only (`#![feature]` on stable) |

**Zero contradictions.** Every crate that built and instantiated its
emitter produced the `mrs RNDR`/`RNDRRS` the classifier predicted.

- `armv8` and `lcpfs` emit both RNDR and RNDRRS directly, confirming the
  `rndr`/`rndrrs` mnemonic split in the classifier maps to distinct
  encodings in the binary (`d53b24 0N` vs `d53b24 2N`).
- `aarch64-cpu`'s `rndr()`/`rndrrs()` are un-called public methods, so the
  bare library object contains no `mrs` -- the same monomorphization limit
  as the x86 wrapper crates (`safe_arch`, `mc-rand`). Not a false positive.
- `cortex-a` is nightly-only and shares `aarch64-cpu`'s lineage; with no
  nightly on the dev host it is build-limited and rests on the static
  verdict plus the `aarch64-cpu` precedent.

## x86 vs ARM, end to end

| | x86 | ARM |
|---|---:|---:|
| in-source crates | 40 | 5 |
| emit-confirmed in binary | 13 | 3 |
| headline compile-time elision | getrandom (SGX: 120 `rdrand`, 0 `cpuid`) | getrandom (rndr backend: 15 `mrs RNDR`, 0 detect) |

The compile-time-elision pattern that defeats CPUID-fault interception on
x86 has a direct AArch64 counterpart, but the affected population is ~8x
smaller (`arm_pass.md`). The interception *contrast* -- MRS sysreg reads
appear directly trappable, unlike RDRAND -- is discussion material and
must be checked against the ARM ARM before assertion.

## Reproducing

```
# on a native aarch64 host, from rust-study/
bash pipeline/verify/arm_verify.sh   # -> results/arm_verify_results.csv
```
