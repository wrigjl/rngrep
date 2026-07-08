#!/usr/bin/env python3
"""Corpus-wide text grep for RDRAND/RDSEED mnemonics across every crate's
.rs source. Broader than the classifier's AST detection: it matches the
mnemonic anywhere in text (macro bodies, string literals, odd paths,
build.rs), so the result minus the classifier's known findings is the
exhaustive candidate pool for macro-hidden / prefilter-missed call sites.

Output: results/expand_grep_raw.csv (name, version, n_rs_matched,
patterns, first_snippet). Reproducible from the pinned snapshot corpus.
"""
import csv, os, sys, tarfile
from multiprocessing import Pool

SCRATCH = os.environ.get("CRATES_SCRATCH_DIR",
                         os.path.expanduser("~/scratch/crates-io"))
TARBALLS = os.path.join(SCRATCH, "tarballs")
MANIFEST = os.path.join(SCRATCH, "manifest.csv")
PATTERNS = [b"rdrand", b"rdseed"]

def shard(name):
    n = name.lower(); L = len(n)
    if L == 0: return "0"
    if L == 1: return "1"
    if L == 2: return "2"
    if L == 3: return f"3/{n[0]}"
    return f"{n[:2]}/{n[2:4]}"

def crate_path(name, ver):
    return os.path.join(TARBALLS, shard(name), name, f"{name}-{ver}.crate")

def scan(row):
    name, ver = row
    p = crate_path(name, ver)
    if not os.path.exists(p):
        return None
    matched_files = 0
    pats = set()
    snippet = ""
    try:
        with tarfile.open(p, "r:gz") as tf:
            for m in tf.getmembers():
                if not m.name.endswith(".rs"):
                    continue
                f = tf.extractfile(m)
                if f is None:
                    continue
                data = f.read().lower()
                hit = [pat for pat in PATTERNS if pat in data]
                if hit:
                    matched_files += 1
                    for h in hit:
                        pats.add(h.decode())
                    if not snippet:
                        # first matching physical line, trimmed
                        idx = min(data.find(pat) for pat in hit if pat in data)
                        ls = data.rfind(b"\n", 0, idx) + 1
                        le = data.find(b"\n", idx)
                        raw = data[ls:(le if le != -1 else len(data))]
                        snippet = raw.decode("utf-8", "replace").strip()[:160]
    except Exception as e:
        return (name, ver, -1, f"ERROR:{type(e).__name__}", "")
    if matched_files == 0:
        return None
    return (name, ver, matched_files, "|".join(sorted(pats)), snippet)

def main():
    rows = []
    with open(MANIFEST) as f:
        r = csv.reader(f); next(r)
        for x in r:
            if len(x) >= 2:
                rows.append((x[0], x[1]))
    print(f"scanning {len(rows)} crates with {os.cpu_count()} workers...",
          file=sys.stderr, flush=True)
    out = []
    errors = 0
    with Pool() as pool:
        for i, res in enumerate(pool.imap_unordered(scan, rows, chunksize=64)):
            if i % 20000 == 0:
                print(f"  {i}/{len(rows)}", file=sys.stderr, flush=True)
            if res is None:
                continue
            if res[2] == -1:
                errors += 1
            out.append(res)
    out.sort(key=lambda t: t[0].lower())
    os.makedirs("results", exist_ok=True)
    with open("results/expand_grep_raw.csv", "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["name", "version", "n_rs_matched", "patterns", "first_snippet"])
        w.writerows(out)
    print(f"done: {len(out)} crates mention rdrand/rdseed in .rs "
          f"({errors} tarball errors)", file=sys.stderr, flush=True)

if __name__ == "__main__":
    main()
