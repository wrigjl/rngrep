# Verifying rngrep's numbers from an archived corpus

## What this is

`rngrep` is the data pipeline behind the paper "Mining RDRAND: A Dataset
of Hardware-Entropy Instruction Use Across the Rust Crate Ecosystem." It
statically analyzes crates.io crates for hardware-entropy instruction use
(x86 RDRAND/RDSEED, AArch64 RNDR/RNDRRS), classifies each call site, and
computes reverse-dependency closures. This document is a procedure for
reproducing one specific snapshot's numbers using an **already-downloaded
copy** of that snapshot's corpus and db-dump (e.g. from this project's
Zenodo archival deposit), without contacting crates.io.

Follow every step below in order, exactly as written. Each step gives an
exact command and an exact expected result. **If an actual result does
not match what is expected, stop at that step.** Do not attempt to
diagnose or fix the mismatch yourself, do not skip ahead, and do not
substitute a different command. Record the exact command you ran and the
exact output you observed, then produce the final report described in
Step 6 with that step marked FAIL and continue no further.

Every step below invokes an actual tool committed in this repository
(`pipeline/crawler`, `pipeline/classifier`, `pipeline/revdep`,
`pipeline/verify/*.sh`) or does a byte-for-byte comparison against
reference files also committed in this repository
(`results/`). No step requires writing new code or
improvising an analysis.

## Required inputs

Before starting, confirm you have all of the following. If anything is
missing, stop and ask for it rather than substituting something else.

1. A git checkout of this repository (`rngrep`), referred to below as
   `$REPO`. Set an environment variable now: `export REPO=<actual path>`.
2. `db-dump.tar.gz`, with expected SHA-256
   `af2a04c11d6fbed99307167d44d28c3360090c8eae186f5bc818cf42f8b0bf36`.
3. The `.crate` corpus, in **one** of these two forms:
   - A single file `corpus.tar`, with its own provided SHA-256 checksum, **or**
   - Ten split files `corpus.tar.part.aa` through `corpus.tar.part.aj`,
     plus a checksum file `corpus.tar.parts.sha256` listing each part's
     SHA-256.
4. At least 60 GB of free disk space at the location you will use for
   staging (50 GB+ for the corpus/db-dump, headroom for build scratch).
5. Network access, needed only in Step 1 below (to fetch this project's
   Rust build dependencies from the crates.io *crate registry* — this is
   a different thing from fetching the `.crate` tarballs under analysis,
   which this procedure never does). No later step needs network access.

## A note on shell state

Some execution environments do **not** preserve shell variables or the
current directory between separate command invocations, even within the
same overall task. This procedure defines four variables
(`$REPO`, `$CRATES_SCRATCH_DIR`, `$CORPUS_DIR`, `$BASE`) and uses them
throughout. **Do not assume an `export` from an earlier step is still in
effect later.** Every command block below that uses one of these
variables repeats its `export` line(s) at the top of that block for
exactly this reason — always run the full block as shown, including the
export lines, even if you believe the variable is already set from a
previous step.

## Step 0: Determine host architecture

Run:
```
uname -m
```
- If the output is exactly `x86_64`: you will use **Branch A** in Step 5.
- If the output is exactly `arm64` or `aarch64`: you will use **Branch B** in Step 5.
- Any other output: stop. Report the exact output. This procedure does
  not cover other architectures.

## Step 1: Verify host configuration and build the pipeline

1.1. Confirm the pinned toolchain resolves correctly:
```
export REPO=<actual path from Required Inputs, step 1>
cd "$REPO" && rustc --version
```
Expected: output starts with `rustc 1.95.0`. If it does not, stop and
report the actual output — do not proceed with a different toolchain
version.

1.2. Build the three pipeline binaries:
```
export REPO=<actual path>
cd "$REPO/pipeline/crawler" && cargo build --release
cd "$REPO/pipeline/classifier" && cargo build --release
cd "$REPO/pipeline/revdep" && cargo build --release
```
Expected: each command exits 0 and its final output line contains the
word `Finished`.

1.3. If Step 0 gave `x86_64`, confirm the SGX target is installed:
```
rustup target list --installed --toolchain 1.95.0
```
Expected output includes both `x86_64-unknown-linux-gnu` and
`x86_64-fortanix-unknown-sgx`. If `x86_64-fortanix-unknown-sgx` is
missing, run `rustup target add x86_64-fortanix-unknown-sgx --toolchain
1.95.0` and re-check; if it is still missing after that, stop and report.

1.4. Confirm a disassembler is present.
- Branch A (`x86_64`): run `objdump --version`. Expected: exits 0. If
  not found, stop and report — do not attempt to install system
  packages yourself.
- Branch B (`arm64`/`aarch64`): run
  `command -v llvm-objdump || ls -la /opt/homebrew/opt/llvm@16/bin/llvm-objdump`.
  Expected: one of these succeeds. If neither does, stop and report.

## Step 2: Stage the corpus and db-dump

2.1. Choose a fresh scratch directory that does not already exist (this
matters — reusing an existing directory that already has partial state
in it can silently produce wrong results). Pick its **parent**
directory on a filesystem with at least 60 GB free — this is where the
corpus, db-dump, and build scratch will actually live, so this is the
disk-space check that matters (not wherever `$REPO` happens to be
checked out).
```
export CRATES_SCRATCH_DIR=<a new path, e.g. ~/scratch_verify>
if [ -e "$CRATES_SCRATCH_DIR" ]; then
  echo "STOP: $CRATES_SCRATCH_DIR already exists"
else
  df -Pk "$(dirname "$CRATES_SCRATCH_DIR")" | tail -1
  mkdir -p "$CRATES_SCRATCH_DIR"
fi
```
If the `STOP` message printed, stop and ask what to do rather than
reusing or deleting the existing directory yourself. From the `df`
line: the 4th column (available space, in 1 KB blocks) must be at
least `62914560` (60 GB). If less, stop and report the actual value —
do not proceed with insufficient space.

2.2. Stage the db-dump:
```
export REPO=<actual path>
export CRATES_SCRATCH_DIR=<same path as step 2.1>
cp <path to provided db-dump.tar.gz> "$CRATES_SCRATCH_DIR/db-dump.tar.gz"
cd "$REPO"
./pipeline/crawler/target/release/crawler --scratch "$CRATES_SCRATCH_DIR" \
  db-dump --expect-sha af2a04c11d6fbed99307167d44d28c3360090c8eae186f5bc818cf42f8b0bf36
```
Expected: exits 0, and the output contains the line
`db-dump sha256 matches the pinned snapshot`. If the output instead
contains the word `Error`, stop and report the exact error text.

2.3. Stage and extract the corpus. `$CORPUS_DIR` is wherever the
provided corpus file(s) actually are — a fourth variable, distinct from
`$CRATES_SCRATCH_DIR`.

If you were given the split form (ten `corpus.tar.part.*` files plus
`corpus.tar.parts.sha256`):
```
export CORPUS_DIR=<directory containing the corpus.tar.part.* files>
cd "$CORPUS_DIR"
shasum -a 256 -c corpus.tar.parts.sha256
```
Expected: ten lines of output, each ending in `OK`. If any line ends in
`FAILED`, stop and report which file(s) failed — do not proceed with a
failed part.

Then extract by streaming the parts straight into `tar` — **do not**
concatenate them into a combined `corpus.tar` file first. That would
briefly need roughly double the disk space (the parts plus a full
second copy, then the extracted tree on top), which has actually run a
staging disk out of space before. Streaming needs only the space the
parts already take plus the extracted tree, not an extra full copy in
between:
```
export CORPUS_DIR=<same path as just above>
export CRATES_SCRATCH_DIR=<same path as step 2.1>
cd "$CORPUS_DIR"
cat corpus.tar.part.* | tar x -C "$CRATES_SCRATCH_DIR"
```
Expected: exits 0. This creates `$CRATES_SCRATCH_DIR/tarballs/`. Skip
ahead to step 2.5.

If you were given a single `corpus.tar` with its own provided checksum
instead, verify then extract it directly:
```
export CORPUS_DIR=<directory containing corpus.tar>
cd "$CORPUS_DIR"
shasum -a 256 corpus.tar
```
Expected: matches the provided checksum exactly. If it does not, stop
and report both values.
```
export CORPUS_DIR=<same path as just above>
export CRATES_SCRATCH_DIR=<same path as step 2.1>
tar xf "$CORPUS_DIR/corpus.tar" -C "$CRATES_SCRATCH_DIR"
```
Expected: exits 0. This creates `$CRATES_SCRATCH_DIR/tarballs/`.

2.5. Verify the extracted file count:
```
export CRATES_SCRATCH_DIR=<same path as step 2.1>
find "$CRATES_SCRATCH_DIR/tarballs" -name '*.crate' | wc -l
```
Expected: exactly `270761`. If the number differs, stop and report the
actual number — do not proceed.

2.6. Rebuild the manifest (this reads and hashes all 270,761 files; it
takes a few minutes, this is expected):
```
export REPO=<actual path>
export CRATES_SCRATCH_DIR=<same path as step 2.1>
cd "$REPO"
./pipeline/crawler/target/release/crawler --scratch "$CRATES_SCRATCH_DIR" manifest-rebuild
```
Expected: exits 0, final output line contains `rebuilt manifest` and
`count=270761`.

2.7. Verify the manifest row count:
```
export CRATES_SCRATCH_DIR=<same path as step 2.1>
tail -n +2 "$CRATES_SCRATCH_DIR/manifest.csv" | wc -l
```
Expected: exactly `270761`.

## Step 3: Classify

All commands from here on use paths relative to `$REPO`. Every block
`cd`s there explicitly first, rather than assuming a previous `cd` is
still in effect.

```
export REPO=<actual path>
export CRATES_SCRATCH_DIR=<path from step 2.1>
cd "$REPO"
mkdir -p verify_out
./pipeline/classifier/target/release/classifier \
  --manifest "$CRATES_SCRATCH_DIR/manifest.csv" \
  --scratch "$CRATES_SCRATCH_DIR" \
  --out verify_out
```
Expected: exits 0, output ends with a line containing
`classification complete`.

Compare each of the four output files against the checked-in reference
(row order may legitimately differ between runs, so each comparison
sorts both sides first):
```
export REPO=<actual path>
cd "$REPO"
diff <(tail -n +2 verify_out/crates.csv | sort) <(tail -n +2 results/crates.csv | sort)
diff <(tail -n +2 verify_out/callsites.csv | sort) <(tail -n +2 results/callsites.csv | sort)
diff <(tail -n +2 verify_out/arm_crates.csv | sort) <(tail -n +2 results/arm_crates.csv | sort)
diff <(tail -n +2 verify_out/arm_callsites.csv | sort) <(tail -n +2 results/arm_callsites.csv | sort)
```
Expected for each: no output, exit 0. Record each as PASS. If any
command prints output, record that comparison as FAIL and include the
full printed output verbatim in your final report.

## Step 4: Closures

```
export REPO=<actual path>
export CRATES_SCRATCH_DIR=<path from step 2.1>
cd "$REPO"
REVDEP=./pipeline/revdep/target/release/revdep
"$REVDEP" --scratch "$CRATES_SCRATCH_DIR" --crates-csv verify_out/crates.csv --out verify_out
"$REVDEP" --scratch "$CRATES_SCRATCH_DIR" --crates-csv verify_out/crates.csv --out verify_out --include-optional
"$REVDEP" --scratch "$CRATES_SCRATCH_DIR" --crates-csv verify_out/arm_crates.csv --out verify_out --wrappers "aarch64-cpu,cortex-a,armv8" --label arm_
"$REVDEP" --scratch "$CRATES_SCRATCH_DIR" --crates-csv verify_out/arm_crates.csv --out verify_out --wrappers "aarch64-cpu,cortex-a,armv8" --label arm_ --include-optional
"$REVDEP" --scratch "$CRATES_SCRATCH_DIR" --crates-csv verify_out/crates.csv --out verify_out --seed-mode build_config --wrappers "" --label bc_
"$REVDEP" --scratch "$CRATES_SCRATCH_DIR" --crates-csv verify_out/arm_crates.csv --out verify_out --seed-mode build_config --wrappers "" --label arm_bc_
```
Expected: each of the six commands exits 0. (`--scratch` is passed
explicitly on every call, rather than relying on `revdep`'s default
`$CRATES_SCRATCH_DIR`-or-`~/scratch/crates-io` fallback, in case that
environment variable isn't set in whatever shell invocation runs this
block — same reasoning as the note at the top of this document.)

Compare the CSV outputs (sorted, same reasoning as Step 3):
```
export REPO=<actual path>
cd "$REPO"
for f in closure.csv closure_maxbound.csv arm_closure.csv arm_closure_maxbound.csv bc_closure.csv arm_bc_closure.csv; do
  echo "=== $f ==="
  diff <(tail -n +2 "verify_out/$f" | sort) <(tail -n +2 "results/$f" | sort)
done
```

Compare the summary reports (plain diff, not sorted — these are fixed-format text, not tables):
```
export REPO=<actual path>
cd "$REPO"
for f in closure_summary.md closure_summary_maxbound.md arm_closure_summary.md arm_closure_summary_maxbound.md bc_closure_summary.md arm_bc_closure_summary.md; do
  echo "=== $f ==="
  diff "verify_out/$f" "results/$f"
done
```
Expected for all twelve comparisons: no output, exit 0. Same PASS/FAIL
recording instruction as Step 3.

## Step 5: Binary verification

Use the branch determined in Step 0.

5.0. Choose a fresh `BASE` directory that does not already exist (same
reasoning as Step 2.1):
```
export BASE=<a new path, e.g. ~/rrv_verify>
if [ -e "$BASE" ]; then
  echo "STOP: $BASE already exists"
else
  mkdir -p "$BASE"
fi
```
If `STOP` printed, stop and ask what to do.

### Branch A (x86_64)

```
export REPO=<actual path>
export CRATES_SCRATCH_DIR=<path from step 2.1>
export BASE=<path from step 5.0>
cd "$REPO"
cp pipeline/verify/verify_candidates_2026-07-08.csv "$BASE/"
CRATES_SCRATCH_DIR="$CRATES_SCRATCH_DIR" TC=1.95.0 BASE="$BASE" \
  CAND="$BASE/verify_candidates_2026-07-08.csv" OUT="$BASE/verify_results.csv" \
  bash pipeline/verify/verify.sh
```
Expected: exits 0, output ends with `ALL DONE`.
```
export REPO=<actual path>
export BASE=<path from step 5.0>
cd "$REPO"
diff <(tail -n +2 "$BASE/verify_results.csv" | sort) <(tail -n +2 results/verify_results.csv | sort)
```
Expected: no output. Record PASS/FAIL accordingly.

Then the refine pass:
```
export REPO=<actual path>
export CRATES_SCRATCH_DIR=<path from step 2.1>
export BASE=<path from step 5.0>
cd "$REPO"
cp pipeline/verify/refine_candidates_2026-07-08.csv "$BASE/"
CRATES_SCRATCH_DIR="$CRATES_SCRATCH_DIR" BASE="$BASE" \
  CAND="$BASE/refine_candidates_2026-07-08.csv" OUT="$BASE/verify_refine.csv" \
  bash pipeline/verify/refine.sh
diff <(tail -n +2 "$BASE/verify_refine.csv" | sort) <(tail -n +2 results/verify_refine.csv | sort)
```
Expected: no output. Record PASS/FAIL accordingly.

### Branch B (arm64/aarch64)

```
export REPO=<actual path>
export CRATES_SCRATCH_DIR=<path from step 2.1>
export BASE=<path from step 5.0>
cd "$REPO"
cp pipeline/verify/arm_candidates_2026-07-08.csv "$BASE/"
CRATES_SCRATCH_DIR="$CRATES_SCRATCH_DIR" TC=1.95.0 BASE="$BASE" \
  CAND="$BASE/arm_candidates_2026-07-08.csv" OUT="$BASE/arm_verify_results.csv" \
  bash pipeline/verify/arm_verify.sh
```
Expected: exits 0, output ends with `ARM VERIFY DONE`.
```
export REPO=<actual path>
export BASE=<path from step 5.0>
cd "$REPO"
diff <(tail -n +2 "$BASE/arm_verify_results.csv" | sort) <(tail -n +2 results/arm_verify_results.csv | sort)
```
Expected: no output. Record PASS/FAIL accordingly.

(There is no ARM equivalent of the x86 refine pass — skip it, this is
expected, not a gap in your run.)

## Step 6: Final report

Produce a report with one row per comparison performed above (18 rows
total on Branch A: 4 in Step 3, 12 in Step 4, 2 in Step 5; 17 rows on
Branch B: 4 + 12 + 1). For each row give: the step number, the exact
command that produced the comparison, PASS or FAIL, and — only for
FAIL rows — the exact output that was printed. Do not add
interpretation, explanation, or speculation about why a FAIL occurred;
report the raw fact.
