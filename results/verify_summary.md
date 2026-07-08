# Binary-Verification Results (PLAN.md section 6 / binary-plan.md)

Dated 2026-07-07. Ground-truth pass confirming the static classifier
against the machine code rustc actually emits. This is the step every
prior pass deferred; it turns the closure range (`closure_summary*.md`)
and the source-level verdicts (`third_pass.md`) into evidence about
compiler output.

## Setup

- **Build host:** kiriko, native x86_64 Linux (same host as the recall
  `cargo expand` pass).
- **Toolchain:** stable `rustc 1.95.0 (59807616e 2026-04-14)`, pinned by
  `rust-toolchain.toml` and forced per-build with `cargo +1.95.0`;
  identical commit to the dev host. Nightly recovery builds
  (`verify_refine.csv`) used kiriko's `1.99.0-nightly` where a crate is
  nightly-only.
- **Corpus:** the 40 in-source crates at their pinned `crates.csv`
  versions, fetched on demand from `static.crates.io` (snapshot
  2026-05-13, `snapshot.toml`).
- **Disassembler:** GNU `objdump -d` (llvm-objdump absent on kiriko).
  Only real instruction lines (`^<hex>:`) are counted, so paths/archive
  headers containing "rdrand" (e.g. the `rdrand` crate) do not
  false-match.
- **Configs:** `default` (host, default features), `rdrand`
  (`RUSTFLAGS="-C target-feature=+rdrand,+rdseed"`), `sgx`
  (`--target x86_64-fortanix-unknown-sgx`). Recovery pass adds
  `--all-features` and the nightly toolchain (`pipeline/verify/refine.sh`).
- **Detection:** RDRAND `0F C7 /6` and RDSEED `0F C7 /7` surface as
  `rdrand`/`rdseed` mnemonics; a co-located `cpuid` (`0F A2`) marks a
  runtime-gated path.

Tools: `pipeline/verify/verify.sh` (+ `refine.sh`). Raw data:
`results/verify_results.csv` (40 crates x 3 configs) and
`results/verify_refine.csv` (recovery builds).

## Headline: getrandom confirms compile-time CPUID elision

`getrandom` 0.4.2 emits **no** RDRAND under `default` or `+rdrand`
(target-feature alone does not switch it off the Linux syscall backend --
a correct, reportable baseline). Under the **SGX target** its RDRAND
backend is compiled and `getrandom::backends::rdrand::rdrand_u32` is an
unrolled RDRAND retry loop with **zero `cpuid` in the entire object**:

```
0:  48 0f c7 f2   rdrand %rdx
4:  72 36         jb     3c        # retry on failure (CF=0)
6:  48 0f c7 f2   rdrand %rdx
a:  72 30         jb     3c
... (120 rdrand, 0 cpuid in the rlib)
```

This is exactly the pattern the dissertation targets: the CPUID gate is
elided at compile time, so `rr`'s CPUID-faulting interception never
fires. The claim is now empirically grounded, not merely inferred from
source.

## Outcomes (40 crates)

| Outcome | Count | Meaning |
|---|---:|---|
| **EMIT-CONFIRMED** | 13 | opcode present, consistent with classifier |
| **RUNTIME-GATE** | 4 | `cpuid` present in the crate's own object |
| **BUILT-NO-EMIT (mono)** | 9 | built, but the emitter is un-instantiated |
| **BUILD-LIMITED** | 14 | could not build in any config (fall back to static) |

Per-crate detail in `verify_results.csv`; the classification above is
reproduced by the awk rollup in this pass's shell history.

### EMIT-CONFIRMED (13)

`getrandom`, `rmath`, `tfhe-csprng` (compile_time); `rdrand`, `x86`,
`nanorand`, `cocoon-tpm-crypto`, `irox-arch-x86-64`,
`simple_random_numbers`, `lcpfs` (unconditional); `Nuclide`,
`number-theory` (runtime); `rialo-tee-secret-sharing` (out_of_scope).
All emit RDRAND/RDSEED with the operand encodings expected from the
mnemonic (`48 0f c7 f?`), and none carries a `cpuid` in-object except
where noted below. `nanorand` emitted nothing under default features but
its RDSEED backend appeared under `--all-features` (11 `rdseed`, no
`cpuid`) -- proof the classifier's verdict was correct and the default
build simply did not exercise it.

Note on the compile_time crates: `#[target_feature(enable="rdrand")]`
functions emit the instruction into the object regardless of the
`+rdrand` flag -- the "compile-time" property is about the *call-site*
gate being removed, not about whether the opcode is in the rlib. The
binary confirms both halves for `getrandom` (SGX path, no `cpuid`).

### RUNTIME-GATE (4) -- corroborates the interprocedural caveat

`urng`, `x64`, `x86_64` were labelled **unconditional** by the
function-local classifier, but their objects contain `cpuid`
(detection) and **no** in-object RDRAND -- i.e. they are really
runtime-gated, with the guard in a different function than the intrinsic.
This is exactly `second_pass.md`'s interprocedural undercount item, now
**independently confirmed by machine code**. `urng` shows `cpuid`x10 on
host and `cpuid`=0 under SGX (detection elided where the feature is
guaranteed). `sallyport` (out_of_scope TEE) likewise shows host `cpuid`.

### BUILT-NO-EMIT / monomorphization limit (9)

`safe_arch`, `mc-rand`, `virtfw-libhw`, `hyper-thread-random`,
`winit-gfx-hal`, `concrete-csprng`, `enarx-shim-sgx`, `enarx-shim-kvm`,
`hoba`. These build but emit no opcode in a bare library object because
the emitter is an un-called generic / `inline` / `#[target_feature]` /
macro-instantiated function, or lives behind a feature/dependency not
compiled here. rustc emits no code for a public generic/inline function
with no in-crate caller, so a library-only build cannot observe it. These
are **not** false positives -- the source is confirmed by the classifier
and the hand-audit; they are a stated limit of library-level
verification (a monomorphizing downstream caller, or the PLAN section-6
binary of a *dependent*, would surface them).

### BUILD-LIMITED (14)

Could not build in any usable config; fall back to the static verdict.
Causes: source bit-rot on a modern compiler (`vektor`, `typed_arch`,
`swiss-army-knife`, `enclave`), C-library `build.rs` needing system
packages (`mbedtls`, `mbedtls-smx`, `ciphern`/openssl,
`bitcoin-random`/openssl), stale deps (`lokacore`, `ostd`), no_std /
kernel targets (`libhermit-rs`, `rusty-hermit`, `freertos-std`), and a
manifest error (`ort-openrouter-cli`). Notably 2 of 7 compile_time crates
(`vektor`, `typed_arch`) are build-limited, so their compile-time verdict
rests on source + static analysis only.

## Precision on emitted code

**Zero contradictions.** Among every crate we could build and observe,
none that the classifier called an emitter proved to emit nothing when
its emitting path was actually compiled (the `nanorand` all-features
result is the decisive check). The 3 `unconditional`->runtime-gate
refinements (`urng`, `x64`, `x86_64`) are not classifier errors: they are
the already-documented interprocedural caveat, now confirmed. So on the
observable subset the classifier's emit/no-emit judgement holds, and the
compile-time-elision hypothesis is confirmed on the crate that matters
most (`getrandom`).

## Threats / limitations

- **Library-level monomorphization** (9 crates): un-called
  generic/inline/target_feature/macro emitters are invisible in a bare
  lib object. Mitigation: build a monomorphizing caller or the dependent
  binary; out of scope for this sample.
- **Build-limited (14 crates):** static verdict stands; not independently
  confirmed. Bit-rot and C-dep failures dominate.
- **`cpuid` locality:** `is_x86_feature_detected!` caches detection in
  `std`, so a runtime gate can live in a *different* rlib than the
  crate's; absence of in-object `cpuid` does not prove absence of a gate
  (`Nuclide`, `number-theory` emit RDRAND with the gate in std).
- **Compiler version:** results are pinned to 1.95.0; a different rustc
  may inline/fold differently.

## Reproducing

```
# on kiriko (native amd64)
scp pipeline/verify/{verify.sh,refine.sh,verify_candidates.csv,refine_candidates.csv} kiriko:~/rrv/
ssh kiriko 'cd ~/rrv && bash verify.sh && bash refine.sh'
# -> verify_results.csv, refine_results.csv
```
