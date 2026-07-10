# Classifier hand-audit results

Manual audit of **all 31 crates** with an in-source RDRAND/RDSEED call
site (80 sites) against the 2026-05-13 snapshot. Each site was checked
against the real source pulled from the `.crate` tarball
(`results/audit_worksheet.md` has the annotated context). This is the
precision/recall ground truth for the `paper.tex` methodology section.

Note: 31, not the headline 30 -- `vektor` is included; the classifier
scored it `none` (all sites flagged `in_test`), so it is absent from the
"30 with a non-`none` category" count. The audit shows that exclusion is
itself an error (see below).

## Verdicts

Legend: OK = correct. OK* = correct under the classifier's stated
per-call-site / function-local rule, but a documented interprocedural
or path caveat applies. ERR = mislabel.

| Crate | Classifier | Audited truth | Verdict |
|---|---|---|---|
| concrete-csprng | compile_time | compile_time | OK* |
| getrandom | compile_time | compile_time | OK |
| rmath | compile_time | compile_time | OK |
| safe_arch | compile_time | **unconditional** | **ERR** |
| tfhe-csprng | compile_time | compile_time | OK* |
| typed_arch | compile_time | compile_time | OK |
| Nuclide | runtime | runtime | OK |
| hoba | runtime | runtime | OK |
| number-theory | runtime | runtime | OK |
| ciphern | unconditional | unconditional | OK* |
| irox-arch-x86-64 | unconditional | unconditional | OK |
| lokacore | unconditional | unconditional | OK |
| mc-rand | unconditional | unconditional | OK |
| rdrand | unconditional | unconditional | OK* |
| simple_random_numbers | unconditional | unconditional | OK |
| swiss-army-knife | unconditional | unconditional | OK |
| urng | unconditional | unconditional | OK* |
| winit-gfx-hal | unconditional | unconditional | OK |
| x64 | unconditional | unconditional | OK* |
| x86 | unconditional | unconditional | OK |
| x86_64 | unconditional | unconditional | OK* |
| enarx-shim-kvm | out_of_scope (sgx) | out_of_scope | OK |
| enarx-shim-sgx | out_of_scope (sgx) | out_of_scope | OK |
| enclave | out_of_scope (sgx) | out_of_scope | OK |
| freertos-std | out_of_scope (sgx) | out_of_scope | OK |
| libhermit-rs | out_of_scope (kernel) | out_of_scope | OK |
| ostd | out_of_scope (kernel) | out_of_scope | OK |
| rusty-hermit | out_of_scope (kernel) | out_of_scope | OK |
| rialo-tee-secret-sharing | out_of_scope (tee) | out_of_scope | OK |
| sallyport | out_of_scope (sgx) | out_of_scope | OK |
| vektor | none | **compile_time** | **ERR** |

Clear errors: **2 / 31** (crate accuracy 93.5% under the function-local
rule). Interprocedural/path caveats (OK*): 6.

## The two errors share one root cause: `#[cfg_attr(...)]` handling

The classifier conflates conditional *attributes* with real gates.

1. **safe_arch (false positive).** Its only "gate" is
   `#[cfg_attr(docsrs, doc(cfg(target_feature = "rdrand")))]` -- a
   documentation attribute active only under `docsrs`, setting
   `doc(cfg(...))` for rustdoc rendering. The actual `pub fn rdrand_u16`
   has no `#[target_feature]` and no real `#[cfg(target_feature)]`; it
   emits the intrinsic unconditionally. The classifier matched the
   `target_feature = "rdrand"` string inside the doc-cfg and set
   `compile_gate_present`. Truth: **unconditional**.

2. **vektor (false negative).** Its fns carry a real
   `#[target_feature(enable = "rdrand")]` (so, `compile_time`) plus
   `#[cfg_attr(test, assert_instr(rdrand))]` (a stdarch test-harness
   attribute). The `in_test_or_bench` detector saw `cfg(test)` in the
   `cfg_attr` and marked every site test-only, collapsing the crate to
   `none`. Truth: **compile_time**.

Fix: when scanning `#[cfg_attr(pred, inner)]`, evaluate `pred`, and
never treat `doc(cfg(...))` payloads as compile gates nor
`cfg_attr(test, ...)` attributes as test-module membership.

## Precision / recall for the headline `compile_time` category

Using audited truth, treating the two errors as the only misclasses:

- Labeled `compile_time`: 6; truly `compile_time`: 5 (all but safe_arch)
  -> **precision 5/6 = 0.833**.
- Truly `compile_time`: 6 (the 5 + vektor, which was missed)
  -> **recall 5/6 = 0.833**.
- **F1 = 0.833.** Both errors trace to one fixable `cfg_attr` bug;
  after the fix the audited sample reaches P = R = 1.0.

`runtime`: 3 labeled, 3 correct (precision 1.0). `out_of_scope`: 9
labeled, 9 correct (validated against name/dependency/metadata signals,
not source gates).

## Interprocedural caveat (the OK* rows), not counted as errors

The function-local rule classifies each call site by its *enclosing*
function's gating. Several crates guard the RDRAND path in a *different*
function, so the per-site label is `unconditional` (or `compile_time`)
while the crate is semantically runtime-checked and rr-interceptable:

- **ciphern** -- caches availability via `is_x86_feature_detected!`
  in an atomic flag; `read_rdrand_u64` itself is unguarded.
- **urng, x64, x86_64** -- contain `is_x86_feature_detected!` guards
  outside the intrinsic wrapper.
- **rdrand** -- `RdRand::new()` performs the CPU feature check; the
  `next_u32`-style methods call the intrinsic unguarded.
- **tfhe-csprng** -- `RdseedSeeder::new()`/`Default` run
  `is_x86_feature_detected!("rdseed")` (CPUID at construction) and
  panic if absent; `seed()` then calls the `#[target_feature]` fn
  unguarded. **concrete-csprng** has the same `is_available()` method
  but nothing on the `seed()` path calls it -- a nice contrast pair.

This is the documented "distant runtime check" undercount
(first_pass.md item 2). It means the `unconditional` bucket is a mild
over-count and the `runtime` bucket a mild under-count at the crate
level; the per-call-site numbers are unaffected. Interprocedural CPUID
dominance analysis is the mitigation; binary verification (PLAN sec.6)
is the ground-truth cross-check.

## Impact on the reverse-dependency closure (fix applied)

The `cfg_attr` bug was fixed in `pipeline/classifier/src/ast.rs`
(`cfg_attr` no longer contributes to the compile-gate stack or the test
marker), the classifier was re-run, and exactly the two predicted crates
moved: `safe_arch` compile_time -> unconditional, `vektor` none ->
compile_time. No other crate changed. The compile_time seed set is now
{getrandom, rmath, typed_arch, concrete-csprng, tfhe-csprng, vektor}.

`revdep` was re-run on the corrected seeds. The headline moved only
slightly (dropping the more-depended-on `safe_arch` seed):

| Closure (lower bound) | before fix | after fix |
|---|---:|---:|
| crates | 34,639 (13.0%) | 34,007 (12.8%) |
| downloads-weighted | 1.87% | 1.84% |

Upper bound is essentially unchanged (186,933 -> 186,907). Post-fix, the
audited sample reaches precision = recall = 1.0 on `compile_time`.

## Reproduction

```
python3 pipeline/audit/extract_audit.py   # regenerates audit_worksheet.md
```
Worksheet lists every site with >>-marked source context; verdicts above
were assigned by reading each.
