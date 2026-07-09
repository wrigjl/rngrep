# results_rerun_2026-07-08

Reference output for a full end-to-end rerun of the pipeline against a
later db-dump snapshot (2026-07-08, sha256
`af2a04c11d6fbed99307167d44d28c3360090c8eae186f5bc818cf42f8b0bf36`),
run with the license-based fetch filter
(`pipeline/crawler/src/license.rs`) in place.

This is **not** the paper's dataset — `../results/` (2026-05-13,
252,202 crates, no license filter) is that, and is unaffected by
anything here. This directory exists as reference output for verifying
the pipeline against a second, later, larger, license-filtered corpus
(270,761 crates). See `../../rust-study/repro-update.md` for full
narrative and the old-vs-new number mapping.

Binary verification (`verify_results.csv`/`verify_refine.csv` for x86,
`arm_verify_results.csv` for ARM) against
`pipeline/verify/verify_candidates_2026-07-08.csv` /
`arm_candidates_2026-07-08.csv`: no contradictions, with one open item
-- `foundation_compact` (a new ARM `compile_time` in-source crate)
shows zero RNDR/RNDRRS in both default and `+rand` configs. On x86, an
analogous case would get an `--all-features` recovery pass via
`refine.sh`; there's no ARM equivalent of that script yet, so this one
is unresolved rather than confirmed either way.

Same file meanings as `../results/` (`docs/SCHEMA.md`), with two
absences: no `expand_*`/`audit_*` files (the manual recall/audit
process wasn't rerun against this snapshot, only classify/closures/
verify) and no `arm_bc_closure`/`bc_closure` narrative beyond the raw
CSVs (not independently reviewed this pass).
