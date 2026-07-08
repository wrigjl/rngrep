#!/usr/bin/env python3
"""Authoritative PRE-expansion detection over the candidate pool.

Expansion only covers crates that build; for the ~1/3 that fail to expand
we still need a verdict. This scans each candidate's raw .rs source for
real RDRAND/RDSEED *emission* the classifier missed:

  - intrinsic call/def:  _rd{rand,seed}{16,32,64}_step
  - intrinsic use-alias:  use ...::_rd..._step as NAME   (top-level, grouped,
                          or function-local -- the classifier gap)
  - inline asm emit:      asm!/global_asm!/llvm_asm! whose template contains
                          the rdrand/rdseed mnemonic

Distinguishes genuine emission from FFI extern decls (bindgen `pub fn
_rdrand.._step(...) -> ...` with no body) which are not rustc-emitted.

Output: results/expand_missed.csv (the confirmed source-visible misses).
"""
import csv, glob, os, re, tarfile

SCRATCH = os.environ.get("CRATES_SCRATCH_DIR",
                         os.path.expanduser("~/scratch/crates-io"))
STEP = re.compile(r'_rd(?:rand|seed)(?:16|32|64)_step')
# an extern FFI decl (bindgen): `pub fn _rdrand16_step(...) -> ...;` no body
FFI_DECL = re.compile(r'pub\s+fn\s+_rd(?:rand|seed)\d+_step\s*\([^)]*\)\s*->[^;{]*;')
USE_ALIAS = re.compile(r'use\s+[^;]*_rd(?:rand|seed)\d+_step\s+as\s+\w+')
ASM = re.compile(r'(?:global_asm|llvm_asm|asm)\s*!\s*\(', re.I)
MNE = re.compile(r'\brd(?:rand|seed)\b', re.I)

def shard(n):
    n=n.lower(); L=len(n)
    return "0" if L==0 else "1" if L==1 else "2" if L==2 else f"3/{n[0]}" if L==3 else f"{n[:2]}/{n[2:4]}"

def cpath(name,ver):
    return os.path.join(SCRATCH,"tarballs",shard(name),name,f"{name}-{ver}.crate")

def scan(name,ver):
    g=glob.glob(cpath(name,ver)) or glob.glob(
        os.path.join(SCRATCH,"tarballs","**",name,f"{name}-{ver}.crate"),recursive=True)
    if not g: return None
    kinds=set(); ev=""
    with tarfile.open(g[0]) as tf:
        for m in tf.getmembers():
            if not m.name.endswith(".rs"): continue
            txt=tf.extractfile(m).read().decode("utf-8","replace")
            inner=m.name.split('/',1)[1]
            for i,l in enumerate(txt.splitlines(),1):
                if USE_ALIAS.search(l):
                    kinds.add("use_alias"); ev=ev or f"{inner}:{i} {l.strip()[:80]}"
                if STEP.search(l) and not FFI_DECL.search(l):
                    # a step token that isn't a bare bindgen extern decl
                    if "extern" not in l and not FFI_DECL.search(l):
                        kinds.add("intrinsic"); ev=ev or f"{inner}:{i} {l.strip()[:80]}"
            # asm emit: any asm! macro whose following text carries the mnemonic
            for mt in ASM.finditer(txt):
                tail=txt[mt.start():mt.start()+400]
                if MNE.search(tail):
                    kinds.add("inline_asm"); ev=ev or f"{inner} asm!(...rd...)"
            # FFI-only (bindgen extern) -> mark so we can exclude
            if FFI_DECL.search(txt) and not kinds:
                kinds.add("ffi_extern_only")
    return kinds, ev

cands=list(csv.DictReader(open("results/expand_candidates.csv")))
missed=[]; ffi_only=[]
for r in cands:
    res=scan(r["name"],r["version"])
    if not res: continue
    kinds,ev=res
    emit={"intrinsic","use_alias","inline_asm"} & kinds
    if emit:
        missed.append((r["name"],r["version"],r["bucket"],"+".join(sorted(emit)),ev))
    elif "ffi_extern_only" in kinds:
        ffi_only.append(r["name"])

missed.sort(key=lambda t:t[0].lower())
with open("results/expand_missed.csv","w",newline="") as f:
    w=csv.writer(f); w.writerow(["name","version","bucket","emit_kind","evidence"])
    w.writerows(missed)

print(f"candidates scanned: {len(cands)}")
print(f"CONFIRMED source-visible misses (real emission): {len(missed)}")
for n,v,b,k,e in missed:
    print(f"  {n:22} {v:12} {k:24} {e[:70]}")
print(f"\nFFI-extern-only (bindgen decls, not rustc-emitted): {len(ffi_only)}")
print("  "+", ".join(ffi_only[:20])+(" ..." if len(ffi_only)>20 else ""))
print("wrote results/expand_missed.csv")
