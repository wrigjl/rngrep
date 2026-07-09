#!/usr/bin/env bash
# Binary-verification driver (binary-plan.md). Runs on kiriko (native
# x86_64). For each in-source crate, at its pinned version, builds under
# three configs and disassembles the emitted object code, confirming the
# static classifier against the machine code rustc actually emits.
#
#   default  : host x86_64-unknown-linux-gnu, default features
#   rdrand   : + RUSTFLAGS="-C target-feature=+rdrand,+rdseed"
#   sgx      : --target x86_64-fortanix-unknown-sgx (compile-time / TEE path)
#
# Detection is GNU objdump (llvm-objdump absent on kiriko); we count only
# real disassembly lines ("^<hex>:") so archive/path strings that contain
# "rdrand" (e.g. the rdrand crate itself) do not false-match. The RDRAND
# 0F C7 /6 and RDSEED 0F C7 /7 opcodes surface as rdrand/rdseed mnemonics;
# a co-located cpuid (0F A2) marks a runtime-gated call path.
#
# Output row per (crate, config):
#   name,version,category,config,target,build,rdrand,rdseed,cpuid,notes
#   build  = ok | fail-lib | fail-bin | fetch-fail
#   rdrand/rdseed/cpuid = instruction counts across all emitted objects
set -u
source "$HOME/.cargo/env" 2>/dev/null || true
export PATH="$HOME/.cargo/bin:$PATH"

TC="${TC:-1.95.0}"                    # pinned toolchain (rust-toolchain.toml)
BASE="${BASE:-$HOME/rrv}"             # scratch (no rdrand/rdseed/cpuid in path)
CAND="${CAND:-$BASE/verify_candidates.csv}"
OUT="${OUT:-$BASE/verify_results.csv}"
SGX_TARGET="x86_64-fortanix-unknown-sgx"
# Read candidate tarballs from the pinned local corpus, not static.crates.io:
# crates.io does not guarantee a given tarball's availability or content
# indefinitely (see docs/corpus.md), so re-fetching live at verify time is
# itself a reproducibility hole -- the whole point of pinning the corpus.
CORPUS="${CRATES_SCRATCH_DIR:-$HOME/scratch/crates-io}"
MANIFEST="$CORPUS/manifest.csv"
mkdir -p "$BASE/logs"
[ -f "$OUT" ] || echo "name,version,category,config,target,build,rdrand,rdseed,cpuid,notes" > "$OUT"

# Mirrors pipeline/crawler/src/config.rs::shard_for -- the crates.io
# sparse-index-style sharding the crawler uses for tarballs/ on disk.
shard_for() {
  local lower len
  lower=$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')
  len=${#lower}
  case "$len" in
    0) echo "0" ;;
    1) echo "1" ;;
    2) echo "2" ;;
    3) echo "3/${lower:0:1}" ;;
    *) echo "${lower:0:2}/${lower:2:2}" ;;
  esac
}

# Disassemble one artifact; echo "rdrand rdseed cpuid" counts (instr lines only).
counts_for() {
  local f="$1"
  objdump -d "$f" 2>/dev/null | grep -E '^[[:space:]]*[0-9a-f]+:[[:space:]]' \
    | awk '{
        for (i=1;i<=NF;i++) {
          if ($i=="rdrand") r++; else if ($i=="rdseed") s++; else if ($i=="cpuid") c++;
        }
      } END { printf "%d %d %d", r+0, s+0, c+0 }'
}

# Sum counts across every rlib/bin/.o under a target subdir.
scan_dir() {
  local dir="$1" cname="$2" f tr=0 ts=0 tc=0
  local libname="lib${cname//-/_}"
  # rlibs for this crate, plus any linked bin, plus emitted .o
  while IFS= read -r f; do
    [ -e "$f" ] || continue
    read r s c < <(counts_for "$f")
    tr=$((tr+r)); ts=$((ts+s)); tc=$((tc+c))
  done < <(find "$dir" \( -name "${libname}-*.rlib" -o -name "*.o" \) 2>/dev/null; \
           find "$dir" -maxdepth 1 -type f -executable 2>/dev/null)
  printf "%d %d %d" "$tr" "$ts" "$tc"
}

build_one() {
  local name="$1" version="$2" cat="$3"
  local d="$BASE/w/$name-$version" log="$BASE/logs/$name.log"
  : > "$log"
  rm -rf "$d"; mkdir -p "$d"
  local shard src_crate
  shard=$(shard_for "$name")
  src_crate="$CORPUS/tarballs/$shard/$name/$name-$version.crate"
  if [ ! -f "$src_crate" ]; then
    for cfg in default rdrand sgx; do
      echo "$name,$version,$cat,$cfg,-,corpus-missing,0,0,0,not in local corpus: $src_crate" >> "$OUT"; done
    rm -rf "$d"; return
  fi
  local expected_sha got_sha
  expected_sha=$(awk -F, -v n="$name" -v v="$version" '$1==n && $2==v {print $3; exit}' "$MANIFEST")
  got_sha=$(sha256sum "$src_crate" | cut -d' ' -f1)
  if [ -n "$expected_sha" ] && [ "$got_sha" != "$expected_sha" ]; then
    for cfg in default rdrand sgx; do
      echo "$name,$version,$cat,$cfg,-,checksum-mismatch,0,0,0,expected $expected_sha got $got_sha" >> "$OUT"; done
    rm -rf "$d"; return
  fi
  cp "$src_crate" "$d/c.crate"
  tar xzf "$d/c.crate" -C "$d" 2>>"$log"
  local src="$d/$name-$version"
  export CARGO_TARGET_DIR="$d/target"
  local libflag="--lib"; [ -f "$src/src/lib.rs" ] || libflag=""

  run_cfg() {
    local cfg="$1" target="$2" rustflags="$3" subdir="$4"
    local ok build note="" r=0 s=0 c=0
    ( cd "$src" && \
      RUSTFLAGS="$rustflags" timeout 600 cargo "+$TC" build --release $libflag \
        ${target:+--target "$target"} >>"$log" 2>&1 )
    if [ $? -eq 0 ]; then
      build=ok
      read r s c < <(scan_dir "$d/target/$subdir/release" "$name")
    else
      [ -n "$libflag" ] && build=fail-lib || build=fail-bin
      # Strip $BASE from captured cargo/rustc error text: some errors (e.g.
      # manifest-parse failures) embed the absolute build path, which is
      # host/run-specific (BASE is chosen per invocation) and would
      # otherwise make an exact-match reproduction check fail for a purely
      # cosmetic reason even when the actual build outcome is identical.
      note=$(grep -m1 -iE '^error' "$log" | tr ',"' ';;' | cut -c1-80 | sed "s#$BASE#<BASE>#g")
    fi
    echo "$name,$version,$cat,$cfg,${target:-host},$build,$r,$s,$c,$note" >> "$OUT"
  }

  run_cfg default ""          ""                                   "."
  run_cfg rdrand  ""          "-C target-feature=+rdrand,+rdseed"  "."
  run_cfg sgx     "$SGX_TARGET" ""                                 "$SGX_TARGET"
  rm -rf "$d"
}

echo "verify: toolchain $TC, out $OUT"
tail -n +2 "$CAND" | while IFS=, read -r name version cat; do
  [ -z "$name" ] && continue
  # Strip a trailing \r: candidates files aren't guaranteed LF-only (e.g.
  # Python's csv.writer default, or Excel-edited CSVs), and a stray \r
  # silently propagates into every output row otherwise (breaks strict CSV
  # parsers even though the raw file looks fine to shell tools like awk).
  cat="${cat%$'\r'}"
  grep -q "^$name,$version," "$OUT" 2>/dev/null && { echo "skip $name"; continue; }
  echo ">> $name $version ($cat)"
  build_one "$name" "$version" "$cat"
done
echo "ALL DONE ($(date -u +%FT%TZ))"
