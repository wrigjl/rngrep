# Dataset Schema

All CSVs are UTF-8 with a header row. The x86 and AArch64 datasets share the
same schemas (`crates.csv`/`arm_crates.csv`, `callsites.csv`/
`arm_callsites.csv`).

## `crates.csv`, `arm_crates.csv` — one row per crate version

| column | type | meaning |
|---|---|---|
| `name`, `version` | str | crate identity (pinned version from the snapshot) |
| `num_callsites` | int | total call sites found in the crate |
| `num_intrinsic` | int | call sites that are intrinsic calls/uses |
| `num_inline_asm` | int | call sites that are inline-asm mnemonics |
| `n_runtime` | int | non-test call sites classified `runtime` |
| `n_compile_gate` | int | non-test call sites classified `compile_time` |
| `n_unconditional` | int | non-test call sites classified `unconditional` |
| `n_out_of_scope` | int | non-test call sites classified `out_of_scope` |
| `n_in_test` | int | call sites inside test/bench code (excluded from category) |
| `category` | str | crate-level worst case (see values below) |
| `build_config_exposure` | bool | crate packaging force-enables the feature via rustflags |
| `build_config_detail` | str | the matched rustflags string(s), `\|`-joined |
| `is_out_of_scope` | bool | crate runs where rr cannot record (SGX/kernel/TEE) |
| `out_of_scope_reason`, `out_of_scope_signals` | str | why, and the signals that fired |
| `parse_failures` | int | `.rs` files that looked relevant but did not parse |

Rows with no call sites appear only when `build_config_exposure = true`.

## `callsites.csv`, `arm_callsites.csv` — one row per call site

| column | type | meaning |
|---|---|---|
| `name`, `version` | str | crate identity |
| `file`, `line` | str/int | location within the crate |
| `kind` | str | `intrinsic` or `inline_asm` |
| `detail` | str | intrinsic name / asm mnemonic (`_rdrand64_step`, `rdseed`, `rndr`, `rndrrs`, `alias:<name>`) |
| `cfg_gates` | str | enclosing `#[cfg(...)]` predicates, `;`-joined, outermost first |
| `in_test_or_bench` | bool | inside test/bench code |
| `runtime_gated` | bool | enclosing fn has `is_x86_feature_detected!` / `is_aarch64_feature_detected!` for this ISA |
| `compile_gate_present` | bool | an enclosing `#[cfg(target_feature=...)]` / SGX gate |
| `target_feature_attr` | bool | enclosing fn has `#[target_feature(enable=...)]` |
| `is_out_of_scope`, `out_of_scope_reason` | bool/str | as in crates.csv |
| `category` | str | per-call-site category (see below) |

### Category values

- `runtime` — guarded by a runtime feature check; `rr` can intercept via CPUID.
- `compile_time` — `#[target_feature(enable=...)]` / `#[cfg(target_feature=...)]`
  / SGX-target gate; the runtime check is elided (the rr-defeating pattern).
- `unconditional` — emitted with no in-crate gate; caller's responsibility.
- `out_of_scope` — runs where rr cannot record (SGX enclave, unikernel/kernel,
  non-SGX TEE).
- `none` — call sites exist only in test/bench code.

Crate-level `category` is the worst case across non-test call sites, ranked
`unconditional > compile_time > runtime > out_of_scope`.

## `closure.csv`, `closure_maxbound.csv`, `arm_closure*`, `bc_closure*`, `arm_bc_closure*`

Reverse-dependency closure of the seed set. One row per reached crate.

| column | type | meaning |
|---|---|---|
| `name` | str | crate name |
| `crate_id` | int | crates.io numeric id |
| `seed_class` | str | `compile_time`, `wrapper`, or `both` |
| `depth` | int | BFS depth from the nearest seed (1 = direct dependent) |
| `direct` | bool | `depth == 1` |
| `downloads` | int | all-time downloads (from the db-dump) |

Files without a suffix are the default-linked **lower bound**; `_maxbound` is
the include-optional **upper bound**. `bc_` / `arm_bc_` are seeded from the
build-config-exposed population instead of the compile-time sources.

## `verify_results.csv`, `verify_refine.csv`, `arm_verify_results.csv`

Binary-verification outcomes: build each in-source crate under several configs
and disassemble.

x86 (`verify_results.csv`): `name, version, category, config, target, build,
rdrand, rdseed, cpuid, notes` — `config` ∈ {default, rdrand, sgx}; `build` ∈
{ok, fail-lib, fail-bin, fetch-fail}; `rdrand/rdseed/cpuid` are instruction
counts in the crate's own objects. `verify_refine.csv` adds a `mode` column
(`nightly` / `allfeat`) for recovery builds.

AArch64 (`arm_verify_results.csv`): `name, version, category, config, build,
rndr, rndrrs, notes` — `config` ∈ {default, rand, backend}.

## `manifest.csv.gz` — corpus reconstruction

`name, version, sha256, size_bytes, fetched_at`. One row per fetched `.crate`.
The `sha256` pins the exact tarball; with `snapshot.toml` this reconstructs the
corpus byte-for-byte from `static.crates.io`.

## Audit / validation artifacts

`audit_results.md`, `audit_worksheet.md` — the precision hand-audit.
`expand_*.csv`, `expand_validation.md` — the macro-expansion (recall)
validation.
