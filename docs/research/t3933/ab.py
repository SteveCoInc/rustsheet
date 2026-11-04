"""T-3933: per-case A/B of two eval reports (before_t3933.json, after_t3933.json in the directory given): cells
changed, visibly off (not equal to 15 significant digits) before/after, cells leaving/entering 15 digits, and nearer/further
from the 50-digit math (<case>.math.json). Usage: python3 docs/research/t3933/ab.py <dir with both reports>"""
import json,sys,collections
from decimal import Decimal as D
from pathlib import Path
S=sys.argv[1]
def load(p):
    r=json.load(open(p)); return {(c["id"],cell["sheet"],cell["cell"]):cell for c in r["cases"] for cell in c.get("cells",[])}
B,A=load(f"{S}/before_t3933.json"),load(f"{S}/after_t3933.json")
def w15(e,x):
    # same 15 significant digits
    return f"{e:.14e}"==f"{x:.14e}"
mathc={}
def math(case,sheet,cell):
    if case not in mathc:
        su,nm=case.split("/",1); p=Path("eval/suites")/su/"cases"/nm.replace(".case.json",".math.json")
        mathc[case]=json.load(open(p))["sheets"] if p.exists() else {}
    v=mathc[case].get(sheet,{}).get(cell)
    return D(v["v"]) if v else None
stats=collections.defaultdict(collections.Counter)
far=[]
for key in set(B)|set(A):
    b,a=B.get(key),A.get(key)
    cell=b or a
    try: e=float(cell["expected"]["v"])
    except: continue
    try:
        xb=float(b["actual"]["v"]) if b else e
        xa=float(a["actual"]["v"]) if a else e
    except (ValueError, TypeError): continue
    if xb==xa: continue
    case=key[0].split("/")[1].replace(".case.json","")
    s=stats[case]
    s["changed"]+=1
    s["vis_before"]+= not w15(e,xb); s["vis_after"]+= not w15(e,xa)
    s["left15"]+= w15(e,xb) and not w15(e,xa)
    s["into15"]+= (not w15(e,xb)) and w15(e,xa)
    m=math(key[0],key[1],key[2])
    if m is not None:
        db,da=abs(D(xb)-m),abs(D(xa)-m)
        if da>db: s["further_from_math"]+=1; far.append((case,key[2],cell["formula"],xb,xa,float(m)))
        elif da<db: s["nearer_math"]+=1
    else: s["no_math"]+=1
for c,s in sorted(stats.items()): print(c,dict(s))
print("further from math examples:",far[:8])
