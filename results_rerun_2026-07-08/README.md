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
(270,761 crates): both x86 and ARM binary verification confirmed zero
contradictions against it. See `../../rust-study/repro-update.md` for
full narrative and the old-vs-new number mapping.

Same file meanings as `../results/` (`docs/SCHEMA.md`), with two
absences: no `expand_*`/`audit_*` files (the manual recall/audit
process wasn't rerun against this snapshot, only classify/closures/
verify) and no `arm_bc_closure`/`bc_closure` narrative beyond the raw
CSVs (not independently reviewed this pass).
