#!/usr/bin/env bash
# Refinement pass over verify.sh. Two recovery modes, per candidate row
# (name,version,category,mode):
#   nightly  - crate is nightly-only (#![feature]); build with the +nightly
#              toolchain it actually requires, not the stable pin.
#   allfeat  - crate built OK on stable but emitted nothing under default
#              features; rebuild with --all-features to tell "feature-gated
#              (classifier correct, just not default)" from a false positive.
# Same detection as verify.sh; results appended to a separate CSV.
set -u
source "$HOME/.cargo/env" 2>/dev/null || true
export PATH="$HOME/.cargo/bin:$PATH"
BASE="${BASE:-$HOME/rrv}"
CAND="${CAND:-$BASE/refine_candidates.csv}"
OUT="${OUT:-$BASE/refine_results.csv}"
SGX_TARGET="x86_64-fortanix-unknown-sgx"
# See verify.sh: read from the pinned local corpus, not static.crates.io.
CORPUS="${CRATES_SCRATCH_DIR:-$HOME/scratch/crates-io}"
MANIFEST="$CORPUS/manifest.csv"
mkdir -p "$BASE/logs2"
[ -f "$OUT" ] || echo "name,version,category,mode,config,target,build,rdrand,rdseed,cpuid,notes" > "$OUT"

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

counts_for() {
  objdump -d "$1" 2>/dev/null | grep -E '^[[:space:]]*[0-9a-f]+:[[:space:]]' \
    | awk '{for(i=1;i<=NF;i++){if($i=="rdrand")r++;else if($i=="rdseed")s++;else if($i=="cpuid")c++}}
           END{printf "%d %d %d",r+0,s+0,c+0}'
}
scan_dir() {
  local dir="$1" cname="$2" f tr=0 ts=0 tc=0 libname="lib${2//-/_}"
  while IFS= read -r f; do [ -e "$f" ] || continue
    read r s c < <(counts_for "$f"); tr=$((tr+r)); ts=$((ts+s)); tc=$((tc+c))
  done < <(find "$dir" \( -name "${libname}-*.rlib" -o -name "*.o" \) 2>/dev/null; \
           find "$dir" -maxdepth 1 -type f -executable 2>/dev/null)
  printf "%d %d %d" "$tr" "$ts" "$tc"
}

refine_one() {
  local name="$1" version="$2" cat="$3" mode="$4"
  local d="$BASE/w2/$name-$version" log="$BASE/logs2/$name.log"; : > "$log"
  rm -rf "$d"; mkdir -p "$d"
  local shard src_crate
  shard=$(shard_for "$name")
  src_crate="$CORPUS/tarballs/$shard/$name/$name-$version.crate"
  [ -f "$src_crate" ] \
    || { echo "$name,$version,$cat,$mode,-,-,corpus-missing,0,0,0,not in local corpus: $src_crate" >> "$OUT"; rm -rf "$d"; return; }
  local expected_sha got_sha
  expected_sha=$(awk -F, -v n="$name" -v v="$version" '$1==n && $2==v {print $3; exit}' "$MANIFEST")
  got_sha=$(sha256sum "$src_crate" | cut -d' ' -f1)
  if [ -n "$expected_sha" ] && [ "$got_sha" != "$expected_sha" ]; then
    echo "$name,$version,$cat,$mode,-,-,checksum-mismatch,0,0,0,expected $expected_sha got $got_sha" >> "$OUT"
    rm -rf "$d"; return
  fi
  cp "$src_crate" "$d/c.crate"
  tar xzf "$d/c.crate" -C "$d" 2>>"$log"
  local src="$d/$name-$version"; export CARGO_TARGET_DIR="$d/target"
  local libflag="--lib"; [ -f "$src/src/lib.rs" ] || libflag=""
  local tc=1.95.0 extra=""
  case "$mode" in nightly) tc=nightly ;; allfeat) extra="--all-features" ;; esac

  run_cfg() {
    local cfg="$1" target="$2" rustflags="$3" subdir="$4" build note="" r=0 s=0 c=0
    ( cd "$src" && RUSTFLAGS="$rustflags" timeout 600 cargo "+$tc" build --release $libflag $extra \
        ${target:+--target "$target"} >>"$log" 2>&1 )
    if [ $? -eq 0 ]; then build=ok; read r s c < <(scan_dir "$d/target/$subdir/release" "$name")
    else [ -n "$libflag" ] && build=fail-lib || build=fail-bin
         note=$(grep -m1 -iE '^error' "$log" | tr ',"' ';;' | sed "s#$BASE#<BASE>#g" | cut -c1-80); fi
    echo "$name,$version,$cat,$mode,$cfg,${target:-host},$build,$r,$s,$c,$note" >> "$OUT"
  }
  run_cfg default ""            ""                                  "."
  run_cfg rdrand  ""            "-C target-feature=+rdrand,+rdseed" "."
  run_cfg sgx     "$SGX_TARGET" ""                                  "$SGX_TARGET"
  rm -rf "$d"
}

tail -n +2 "$CAND" | while IFS=, read -r name version cat mode; do
  [ -z "$name" ] && continue
  mode="${mode%$'\r'}"  # see verify.sh: strip stray \r from non-LF-only candidates files
  echo ">> $name ($mode)"; refine_one "$name" "$version" "$cat" "$mode"
done
echo "REFINE DONE ($(date -u +%FT%TZ))"
