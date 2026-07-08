#!/usr/bin/env python3
"""Refine the broad corpus grep: re-scan the crates that matched the loose
substring net with a *word-boundary* regex, killing compound-identifier
false positives (keywo|rdseed|, reco|rdrand|omizer, sha|rdrand|om, ...).

Strict pattern matches the meaningful forms:
  _rdrand / _rdseed        -> intrinsic step fns, has_rdrand, wrappers
  \\brdrand\\b / \\brdseed\\b  -> asm mnemonics and "rdrand"/"rdseed" literals
                              (is_x86_feature_detected, target_feature, cfg)

Reads results/expand_grep_raw.csv (the 293 loose hits), re-derives the
candidate pool against classifier-known crates, writes
results/expand_candidates.csv.
"""
import csv, os, re, tarfile

SCRATCH = os.environ.get("CRATES_SCRATCH_DIR",
                         os.path.expanduser("~/scratch/crates-io"))
TARBALLS = os.path.join(SCRATCH, "tarballs")
STRICT = re.compile(rb"_rdrand|_rdseed|\brdrand\b|\brdseed\b")

def shard(name):
    n = name.lower(); L = len(n)
    if L == 0: return "0"
    if L == 1: return "1"
    if L == 2: return "2"
    if L == 3: return f"3/{n[0]}"
    return f"{n[:2]}/{n[2:4]}"

def crate_path(name, ver):
    return os.path.join(TARBALLS, shard(name), name, f"{name}-{ver}.crate")

def strict_scan(name, ver):
    p = crate_path(name, ver)
    files = 0; snippet = ""
    with tarfile.open(p, "r:gz") as tf:
        for m in tf.getmembers():
            if not m.name.endswith(".rs"):
                continue
            f = tf.extractfile(m)
            if f is None: continue
            data = f.read().lower()
            mo = STRICT.search(data)
            if mo:
                files += 1
                if not snippet:
                    idx = mo.start()
                    ls = data.rfind(b"\n", 0, idx) + 1
                    le = data.find(b"\n", idx)
                    raw = data[ls:(le if le != -1 else len(data))]
                    snippet = raw.decode("utf-8", "replace").strip()[:160]
    return files, snippet

# classifier-known crates
known, insrc = set(), set()
for r in csv.DictReader(open("results/crates.csv")):
    k = (r["name"], r["version"]); known.add(k)
    if int(r["num_callsites"]) > 0: insrc.add(k)

loose = list(csv.DictReader(open("results/expand_grep_raw.csv")))
strict_hits = []
for r in loose:
    k = (r["name"], r["version"])
    files, snip = strict_scan(*k)
    if files > 0:
        strict_hits.append((k, files, snip))

kept = {k for k, _, _ in strict_hits}
dropped = len(loose) - len(strict_hits)
cand = [(k, f, s) for k, f, s in strict_hits if k not in known]
cand.sort(key=lambda t: (-t[1], t[0][0].lower()))

with open("results/expand_candidates.csv", "w", newline="") as fh:
    w = csv.writer(fh)
    w.writerow(["name", "version", "n_rs_strict", "first_snippet"])
    for (n, v), f, s in cand:
        w.writerow([n, v, f, s])

print(f"loose hits: {len(loose)}")
print(f"strict hits (word-boundary): {len(strict_hits)}  (dropped {dropped} substring-noise)")
print(f"in-source known still strict-matching (sanity, expect 31): "
      f"{len({k for k,_,_ in strict_hits if k in insrc})}")
print(f"CANDIDATE POOL (strict, not classifier-known): {len(cand)}")
print("wrote results/expand_candidates.csv")
print("\n--- candidate pool ---")
for (n, v), f, s in cand:
    print(f"  {n:34} {v:12} files={f:>2}  {s[:66]}")
