#!/usr/bin/env bash
# ARM binary-verification driver (binary-plan.md, cross-ISA). Runs on a
# native aarch64 host (the dev host, aarch64-apple-darwin). For each ARM
# in-source crate, build under two/three configs and disassemble, confirming
# the classifier against the emitted MRS reads of the RNDR/RNDRRS system
# registers.
#
#   default : native aarch64, default features
#   rand    : + RUSTFLAGS="-C target-feature=+rand"
#   backend : getrandom only -- select the rndr backend
#             (RUSTFLAGS='--cfg getrandom_backend="rndr" -C target-feature=+rand')
#
# RNDR/RNDRRS have no standalone instruction; both are `mrs <Xt>, RNDR|RNDRRS`
# (d5 3b 24 0N / d5 3b 24 2N). We count llvm-objdump instruction lines whose
# mnemonic is `mrs` and operand is rndr/rndrrs, so symbol/path strings that
# contain "rndr" (e.g. getrandom's rndr.rs) do not false-match.
#
# Output row per (crate, config):
#   name,version,category,config,build,rndr,rndrrs,notes
set -u
source "$HOME/.cargo/env" 2>/dev/null || true
TC="${TC:-1.95.0}"
OBJDUMP="${OBJDUMP:-/opt/homebrew/opt/llvm@16/bin/llvm-objdump}"
BASE="${BASE:-$HOME/rrv}"
CAND="${CAND:-$BASE/arm_candidates.csv}"
OUT="${OUT:-$BASE/arm_verify_results.csv}"
# See pipeline/verify/verify.sh: read from the pinned local corpus, not
# static.crates.io -- crates.io does not guarantee tarball availability or
# content indefinitely, so a live re-fetch at verify time defeats the point
# of pinning the corpus.
CORPUS="${CRATES_SCRATCH_DIR:-$HOME/scratch/crates-io}"
MANIFEST="$CORPUS/manifest.csv"
mkdir -p "$BASE/logs"
echo "name,version,category,config,build,rndr,rndrrs,notes" > "$OUT"

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

# Count mrs rndr / rndrrs across every rlib/bin/.o under a dir.
counts_for() {
  "$OBJDUMP" -d "$1" 2>/dev/null | grep -iE '[[:space:]]mrs[[:space:]].*rndr' \
    | awk '{ if (tolower($0) ~ /rndrrs/) s++; else r++ } END { printf "%d %d", r+0, s+0 }'
}
scan_dir() {
  local dir="$1" cname="$2" f tr=0 ts=0 libname="lib${2//-/_}"
  while IFS= read -r f; do [ -e "$f" ] || continue
    read r s < <(counts_for "$f"); tr=$((tr+r)); ts=$((ts+s))
  done < <(find "$dir" \( -name "${libname}-*.rlib" -o -name "*.o" \) 2>/dev/null; \
           find "$dir" -maxdepth 1 -type f -perm +111 2>/dev/null)
  printf "%d %d" "$tr" "$ts"
}

build_one() {
  local name="$1" version="$2" cat="$3"
  local d="$BASE/w/$name-$version" log="$BASE/logs/$name.log"; : > "$log"
  rm -rf "$d"; mkdir -p "$d"
  local shard src_crate
  shard=$(shard_for "$name")
  src_crate="$CORPUS/tarballs/$shard/$name/$name-$version.crate"
  [ -f "$src_crate" ] \
    || { echo "$name,$version,$cat,default,corpus-missing,0,0,not in local corpus: $src_crate" >> "$OUT"; return; }
  local expected_sha got_sha
  expected_sha=$(awk -F, -v n="$name" -v v="$version" '$1==n && $2==v {print $3; exit}' "$MANIFEST")
  got_sha=$(shasum -a 256 "$src_crate" | cut -d' ' -f1)
  if [ -n "$expected_sha" ] && [ "$got_sha" != "$expected_sha" ]; then
    echo "$name,$version,$cat,default,checksum-mismatch,0,0,expected $expected_sha got $got_sha" >> "$OUT"
    return
  fi
  cp "$src_crate" "$d/c.crate"
  tar xzf "$d/c.crate" -C "$d" 2>>"$log"
  local src="$d/$name-$version"; export CARGO_TARGET_DIR="$d/target"
  local libflag="--lib"; [ -f "$src/src/lib.rs" ] || libflag=""

  run_cfg() {
    local cfg="$1" rustflags="$2" build note="" r=0 s=0
    ( cd "$src" && RUSTFLAGS="$rustflags" timeout 600 cargo "+$TC" build --release $libflag >>"$log" 2>&1 )
    if [ $? -eq 0 ]; then build=ok; read r s < <(scan_dir "$d/target/release" "$name")
    else [ -n "$libflag" ] && build=fail-lib || build=fail-bin
         note=$(grep -m1 -iE '^error' "$log" | tr ',"' ';;' | sed "s#$BASE#<BASE>#g" | cut -c1-70); fi
    echo "$name,$version,$cat,$cfg,$build,$r,$s,$note" >> "$OUT"
  }
  run_cfg default ""
  run_cfg rand    "-C target-feature=+rand"
  if [ "$name" = "getrandom" ]; then
    run_cfg backend '--cfg getrandom_backend="rndr" -C target-feature=+rand'
  fi
  rm -rf "$d"
}

tail -n +2 "$CAND" | while IFS=, read -r name version cat; do
  [ -z "$name" ] && continue
  cat="${cat%$'\r'}"  # see verify.sh: strip stray \r from non-LF-only candidates files
  echo ">> $name $version ($cat)"; build_one "$name" "$version" "$cat"
done
echo "ARM VERIFY DONE ($(date -u +%FT%TZ))"
