#!/usr/bin/env python3
"""Build the hand-audit worksheet: for every flagged call site, pull the real
source context out of the .crate tarball on scratch."""
import csv, os, tarfile, sys
from collections import defaultdict

SCRATCH = os.environ.get("CRATES_SCRATCH_DIR",
                         os.path.expanduser("~/scratch/crates-io"))
TARBALLS = os.path.join(SCRATCH, "tarballs")
CTX = 6  # lines of context on each side

def shard(name):
    n = name.lower()
    L = len(n)
    if L == 0: return "0"
    if L == 1: return "1"
    if L == 2: return "2"
    if L == 3: return f"3/{n[0]}"
    return f"{n[:2]}/{n[2:4]}"

def crate_path(name, ver):
    return os.path.join(TARBALLS, shard(name), name, f"{name}-{ver}.crate")

# group call sites by crate
sites = defaultdict(list)
meta = {}
with open("results/callsites.csv") as f:
    for r in csv.DictReader(f):
        key = (r["name"], r["version"])
        sites[key].append(r)
        meta[key] = r

# also pull crate-level category + oos reason
crate_cat = {}
with open("results/crates.csv") as f:
    for r in csv.DictReader(f):
        crate_cat[(r["name"], r["version"])] = r

def read_file_from_crate(path, wanted):
    """Return dict: inner-file -> list of lines, for files we need."""
    out = {}
    if not os.path.exists(path):
        return None
    with tarfile.open(path, "r:gz") as tf:
        for m in tf.getmembers():
            # inner paths look like `<name>-<ver>/src/...`
            inner = m.name.split("/", 1)[1] if "/" in m.name else m.name
            if inner in wanted:
                fobj = tf.extractfile(m)
                if fobj:
                    txt = fobj.read().decode("utf-8", "replace")
                    out[inner] = txt.splitlines()
    return out

lines_out = []
w = lines_out.append
w("# RDRAND classifier hand-audit worksheet\n")
w("Snapshot 2026-05-13. One block per crate; verify each flagged site "
  "against the source context, then fill the VERDICT.\n")
w("Verdict legend: OK (category correct) | WRONG->X (should be X) | "
  "NOTE (correct but caveat).\n")

missing = []
for (name, ver) in sorted(sites, key=lambda k: (crate_cat.get(k,{}).get("category",""), k[0].lower())):
    cc = crate_cat.get((name, ver), {})
    cat = cc.get("category","?")
    reason = cc.get("out_of_scope_reason","")
    oos_sig = cc.get("out_of_scope_signals","")
    path = crate_path(name, ver)
    wanted = {s["file"] for s in sites[(name, ver)]}
    files = read_file_from_crate(path, wanted)
    w("\n" + "="*70)
    w(f"## {name} {ver}  --  classifier category: **{cat}**"
      + (f"  (oos_reason={reason})" if reason else ""))
    if oos_sig:
        w(f"oos_signals: {oos_sig}")
    if files is None:
        w(f"  !! tarball missing: {path}")
        missing.append((name, ver))
        w(f"\nVERDICT: ____")
        continue
    for s in sites[(name, ver)]:
        fn, ln = s["file"], int(s["line"])
        w(f"\n--- {fn}:{ln}  kind={s['kind']} detail={s['detail']} "
          f"site_cat={s['category']} in_test={s['in_test_or_bench']} "
          f"rt_gated={s['runtime_gated']} cg_present={s['compile_gate_present']} "
          f"tf_attr={s['target_feature_attr']}")
        if s["cfg_gates"]:
            w(f"    cfg_gates: {s['cfg_gates']}")
        src = files.get(fn)
        if not src:
            w(f"    !! file {fn} not in tarball")
            continue
        lo = max(0, ln-1-CTX); hi = min(len(src), ln+CTX)
        for i in range(lo, hi):
            marker = ">>" if (i+1)==ln else "  "
            w(f"    {marker}{i+1:5} {src[i]}")
    w(f"\nVERDICT: ____")

w("\n\n" + "="*70)
w(f"\nMissing tarballs: {len(missing)}")
for m in missing: w(f"  {m}")

open("results/audit_worksheet.md","w").write("\n".join(lines_out))
print(f"crates: {len(sites)}  sites: {sum(len(v) for v in sites.values())}  "
      f"missing tarballs: {len(missing)}")
print("wrote results/audit_worksheet.md")
