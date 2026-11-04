"""T-3580: score the engine on a probes-numeric F case. usage: score.py CASE [OUT.json] (exact/w4/w15 per region).
The binary is $RUSTSHEET_EVAL or target/release/rustsheet-eval; regions come from CASE.meta.json here or in
../t2464/r5-bgrat-fdf1."""
import json, math, os, re, struct, subprocess, sys
from collections import defaultdict
from pathlib import Path
HERE = Path(__file__).resolve().parent
R = str(HERE.parents[2])
case=sys.argv[1] if len(sys.argv)>1 else "t3567-r5-fdf1-hold"
out=subprocess.run([os.environ.get("RUSTSHEET_EVAL", R + "/target/release/rustsheet-eval"),"show","--all",f"probes-numeric/{case}.case.json"],capture_output=True,text=True,cwd=R).stdout
cells={}
cur=None
for line in out.splitlines():
    m=re.match(r"Calc!(\w+)\s+(\S+)",line)
    if m: cur=m.group(1); cells[cur]={"st":m.group(2)}; continue
    m=re.match(r"\s+(expected|actual)\s*: (.*)",line)
    if m and cur: cells[cur][m.group(1)]=m.group(2)
def ordk(v):
    b=struct.unpack("<q",struct.pack("<d",v))[0]
    return b if b>=0 else -(b & 0x7fffffffffffffff)
def tally(pairs):
    ex=w4=w15=0
    for a,e in pairs:
        try: a=float(a); e=float(e)
        except: continue
        if a==e: ex+=1
        if math.isfinite(a) and abs(ordk(a)-ordk(e))<=4: w4+=1
        if e==0: w15+= a==0
        elif abs(a-e)<=0.5*10**(math.floor(math.log10(abs(e)))-14): w15+=1
    return f"{ex}/{w4}/{w15}"
meta=None
for d in (HERE, HERE.parent / "t2464/r5-bgrat-fdf1"):
    if (d / f"{case}.meta.json").exists():
        meta = json.load(open(d / f"{case}.meta.json"))
g=defaultdict(list)
regmap={}
if meta:
    for k,reg,*_ in meta:
        regmap[f"A{k}"]=reg; regmap[f"B{k}"]=reg
for c,d in cells.items():
    p=(d.get("actual"),d.get("expected"))
    g["all"].append(p); g[regmap.get(c,"?")].append(p)
for k in sorted(g): print(k,len(g[k]),tally(g[k]))
if len(sys.argv)>2:
    json.dump({c:d.get("actual") for c,d in cells.items()},open(sys.argv[2],"w"))
