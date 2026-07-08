#!/usr/bin/env bash
#
# End-to-end reproduction driver for rngrep.
#
# Stages are ordered and mostly independent. The corpus fetch (stage 1) is
# heavy (~34 GB of .crate tarballs + ~5 GB db-dump, ~40 GB total) and is
# pinned by snapshot.toml + results/manifest.csv.gz. Tarballs are streamed
# through a gzip decoder in memory by the classifier and never extracted to
# disk, so there is no large decompressed working set (see REPRODUCE.md).
# Binary verification (stage 4) must run on a native host per ISA and is
# left as an explicit, manual step.
#
# The pinned toolchain (rust-toolchain.toml -> rustc 1.95.0) is selected
# automatically inside this directory. Re-running the recall/macro-expansion
# validation needs nightly: `cargo +nightly ...` (see pipeline/audit).
set -euo pipefail
cd "$(dirname "$0")"

echo "== rngrep reproduce =="
rustc --version

# --- Stage 0: build the pipeline ------------------------------------------
echo "== build pipeline =="
for c in crawler classifier revdep; do
  ( cd "pipeline/$c" && cargo build --release )
done

# --- Stage 1: corpus (heavy; skip with SKIP_CORPUS=1 to reuse a scratch) ---
# Fetches the db-dump pinned in snapshot.toml and the latest non-yanked
# .crate for each crate. Set CRATES_SCRATCH_DIR to control where it lands
# (defaults to ~/scratch/crates-io). Skips if the manifest already exists.
if [ "${SKIP_CORPUS:-0}" != "1" ]; then
  echo "== fetch corpus (this is large) =="
  # Enforce the committed db-dump pin. crates.io serves only the latest dump,
  # so reproducing the snapshot means staging the archived db-dump.tar.gz in
  # the scratch dir first (see REPRODUCE.md); --expect-sha then verifies it.
  PINNED_SHA=$(grep -E '^sha256' snapshot.toml | head -1 | cut -d'"' -f2)
  pipeline/crawler/target/release/crawler db-dump --expect-sha "$PINNED_SHA"
  pipeline/crawler/target/release/crawler fetch
else
  echo "== SKIP_CORPUS=1: reusing existing corpus scratch =="
fi

# --- Stage 2: classify the whole corpus -----------------------------------
# Writes results/{crates,callsites}.csv (x86) and results/arm_*.csv (AArch64).
echo "== classify =="
pipeline/classifier/target/release/classifier --out results/

# --- Stage 3: reverse-dependency closures ---------------------------------
echo "== closures =="
REVDEP=pipeline/revdep/target/release/revdep
$REVDEP                                          # x86 compile-time, lower bound
$REVDEP --include-optional                        # x86 compile-time, upper bound
$REVDEP --crates-csv results/arm_crates.csv \
        --wrappers "aarch64-cpu,cortex-a,armv8" --label arm_
$REVDEP --crates-csv results/arm_crates.csv \
        --wrappers "aarch64-cpu,cortex-a,armv8" --label arm_ --include-optional
$REVDEP --seed-mode build_config --wrappers "" --label bc_
$REVDEP --crates-csv results/arm_crates.csv \
        --seed-mode build_config --wrappers "" --label arm_bc_

# --- Stage 4: binary verification (manual, per-ISA native host) -----------
cat <<'NOTE'
== binary verification (manual) ==
Run on a native host per ISA (disassembly only; no privileged hardware needed):
  x86_64  : bash pipeline/verify/verify.sh ; bash pipeline/verify/refine.sh
  aarch64 : bash pipeline/verify/arm_verify.sh
Outputs: results/verify_results.csv, results/verify_refine.csv,
         results/arm_verify_results.csv
See results/verify_summary.md and results/arm_verify_summary.md.
NOTE

echo "== done: see results/ and docs/SCHEMA.md =="
