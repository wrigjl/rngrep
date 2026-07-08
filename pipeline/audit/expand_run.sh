#!/usr/bin/env bash
# cargo-expand validation driver (runs on kiriko, native x86_64).
# For each candidate crate: fetch from static.crates.io, run the expand
# fallback ladder (all-features -> targeted rdrand/rdseed features ->
# default), then detect RDRAND/RDSEED intrinsics in the expanded output.
#
# Output row per crate:
#   name,version,bucket,config_used,expand_ok,pre_step,post_step,post_asm,sample
#   pre_step  = files in raw source literally containing _rd{rand,seed}NN_step
#   post_step = matches of that intrinsic in the EXPANDED output
#   post_asm  = quoted "rdrand/"rdseed occurrences (asm templates / literals)
# A confirmed macro-hidden emitter = post_step>0 while pre_step==0.
set -u
source "$HOME/.cargo/env" 2>/dev/null || true
export PATH="$HOME/.cargo/bin:$PATH"

BASE="$HOME/rdrand-expand"
WORK="$BASE/work"; LOGS="$BASE/logs"
OUT="${OUT:-$BASE/expand_results.csv}"; CAND="${CAND:-$BASE/expand_candidates.csv}"
PAR="${PAR:-6}"
export CARGO_HOME="$HOME/.cargo"
STEP_RE='_rd(rand|seed)(16|32|64)_step'
mkdir -p "$WORK" "$LOGS"
[ -f "$OUT" ] || echo "name,version,bucket,config_used,expand_ok,pre_step,post_step,post_asm,sample" > "$OUT"

expand_one() {
  local name="$1" version="$2" bucket="$3"
  # resume: skip if already recorded
  grep -q "^$name,$version," "$OUT" 2>/dev/null && return
  local d="$WORK/$name-$version" log="$LOGS/$name.log"
  rm -rf "$d"; mkdir -p "$d"
  if ! curl -fsSL "https://static.crates.io/crates/$name/$name-$version.crate" \
        -o "$d/c.crate" 2>>"$log"; then
    echo "$name,$version,$bucket,FETCH_FAIL,0,0,0,0," >> "$OUT"; rm -rf "$d"; return
  fi
  tar xzf "$d/c.crate" -C "$d" 2>>"$log"
  local src="$d/$name-$version"
  [ -d "$src" ] || { echo "$name,$version,$bucket,UNPACK_FAIL,0,0,0,0," >> "$OUT"; rm -rf "$d"; return; }
  export CARGO_TARGET_DIR="$d/target"
  local pre; pre=$(grep -rlE "$STEP_RE" --include=*.rs "$src" 2>/dev/null | wc -l | tr -d ' ')
  local tgt="--lib"; [ -f "$src/src/lib.rs" ] || tgt=""
  # targeted features: [features] keys mentioning rdrand/rdseed
  local feats
  feats=$(awk '/^\[features\]/{f=1;next} /^\[/{f=0} f&&/rdrand|rdseed/{split($0,a,"=");gsub(/[ "\t]/,"",a[1]);print a[1]}' \
            "$src/Cargo.toml" 2>/dev/null | grep . | paste -sd, -)
  local ok="" cfgused="FAILED" exp="$d/expanded.rs"
  ( cd "$src" || exit 1
    for attempt in all targeted default; do
      case $attempt in
        all)      set -- --all-features ;;
        targeted) [ -n "$feats" ] || continue; set -- --features "$feats" ;;
        default)  set -- ;;
      esac
      if timeout 360 cargo expand $tgt "$@" >"$exp" 2>>"$log" && [ -s "$exp" ]; then
        echo "$attempt" > "$d/.cfg"; break
      fi
    done )
  [ -f "$d/.cfg" ] && { ok=1; cfgused=$(cat "$d/.cfg"); }
  if [ -z "$ok" ]; then
    echo "$name,$version,$bucket,FAILED,0,$pre,0,0," >> "$OUT"
    rm -rf "$d"; return
  fi
  local post asm sample
  post=$(grep -cE "$STEP_RE" "$exp" 2>/dev/null | tr -d ' ')
  asm=$(grep -coE '"rd(rand|seed)' "$exp" 2>/dev/null | tr -d ' ')
  sample=$(grep -oE "$STEP_RE" "$exp" 2>/dev/null | head -1)
  echo "$name,$version,$bucket,$cfgused,1,$pre,$post,$asm,$sample" >> "$OUT"
  # keep expanded output only when it actually has a hit (saves space)
  if [ "${post:-0}" -gt 0 ] || [ "${asm:-0}" -gt 0 ]; then
    cp "$exp" "$BASE/hits/$name-$version.expanded.rs" 2>/dev/null
  fi
  rm -rf "$d"
}
export -f expand_one
export OUT WORK LOGS STEP_RE CARGO_HOME BASE PATH

mkdir -p "$BASE/hits"
tail -n +2 "$CAND" | awk -F, '{print $1, $2, $4}' \
  | xargs -P "$PAR" -L1 bash -c 'expand_one "$0" "$1" "$2"'
echo "ALL DONE ($(date -u +%FT%TZ))"
