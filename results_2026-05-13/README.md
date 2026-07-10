# results_2026-05-13/ — historical (superseded)

Pipeline output against the **2026-05-13** crates.io db-dump snapshot
(sha256 `db6bf2ac1b1217d627bcae1594d0f78e32ded593eb128a6fbf174efb2e71e974`),
corpus of **252,202** non-yanked, non-prerelease crate-versions with
**no license filter**.

**This is not the canonical dataset.** The paper is on the 2026-07-08
snapshot; see `../results/`. This directory is retained to document the
earlier work — the classify / closure / precision-audit / recall-audit /
binary-verification passes that were done on the May snapshot.

**The May source corpus is not redistributable.** It was fetched before
the license-based fetch filter existed, so it includes crate-versions
whose licenses do not permit unmodified redistribution. Only these
metadata- and source-*derived* results are kept here; the underlying
`.crate` corpus is **not** part of the archival deposit (unlike the July
corpus, which is redistributable by construction). The July rerun
reproduced the headline findings on a fresh, larger, redistributable
corpus — see `../results/` and `../../rust-study/repro-update.md`.

File meanings: `../docs/SCHEMA.md`.
