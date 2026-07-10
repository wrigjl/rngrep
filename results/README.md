# results/ — the canonical dataset

Reference output of the pipeline against the **canonical** crates.io
db-dump snapshot (**2026-07-08**, sha256
`af2a04c11d6fbed99307167d44d28c3360090c8eae186f5bc818cf42f8b0bf36`),
pinned in `../snapshot.toml`. This is **the paper's dataset**: every
number in the paper refers to this snapshot. Corpus is **270,761**
non-yanked, redistributably licensed crate-versions, built with the
license-based fetch filter (`pipeline/crawler/src/license.rs`) applied
during target selection, so the source corpus is redistributable by
construction. See `../../rust-study/repro-update.md` for the full
old-vs-new narrative and the May→July number mapping.

File meanings are documented in `../docs/SCHEMA.md`.

## Validation for this snapshot

- **Recall** (`expand_validation.md` + `expand_grep_raw.csv` /
  `expand_candidates.csv` / `expand_missed.csv`): exhaustive corpus grep
  → 178-crate candidate pool → pre-expansion source scan; source-visible
  miss set empty (only `core_arch`/`coresimd` intrinsic definitions,
  excluded like the wrapper libraries).
- **Precision** (`audit_results.md` + `audit_worksheet.md`): emit/no-emit
  sound on both ISAs; the 10 headline `compile_time` crates verified as
  genuine `#[target_feature]` gates; interprocedural over-count still
  exactly 3 crates (`urng`/`x64`/`x86_64`).
- **Binary verification** (`verify_results.csv`/`verify_refine.csv` x86,
  `arm_verify_results.csv` ARM) against
  `pipeline/verify/verify_candidates_2026-07-08.csv` /
  `arm_candidates_2026-07-08.csv`: no contradictions. One open item —
  `foundation_compact` (a new ARM `compile_time` in-source crate) shows
  zero RNDR/RNDRRS in both default and `+rand` configs; there is no ARM
  equivalent of x86's `--all-features` `refine.sh` recovery pass yet, so
  this one is unresolved rather than confirmed either way.

Not re-run for this snapshot: the `cargo expand` macro-generation
cross-check (`expand_run.sh`, needs the x86 nightly host) — the
pre-expansion source scan stands in (see `expand_validation.md`); and
the `*_summary.md` narrative rollups / `arm_bc_closure`/`bc_closure`
prose beyond the raw CSVs.

## Historical May pass — `../results_2026-05-13/`

The earlier 2026-05-13 pass (252,202 crates, **no** license filter) is
retained under `../results_2026-05-13/` to document the work that was
done. Its metadata-derived outputs are kept, but the **May source corpus
is not redistributable** (it predates the license filter), so only those
derived results — not that corpus — are archived. July is canonical; the
May directory is superseded.
