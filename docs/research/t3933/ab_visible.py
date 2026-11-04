"""T-3933: per-case A/B by the eval's own visibly-off rule (decision 0083/0088: more than 4 ulp and more than half a
unit in the 15th digit), nearer/further from Excel and from the 50-digit math. Usage as ab.py."""
import json,sys,collections,math as M,struct
from decimal import Decimal as D
from pathlib import Path
S=sys.argv[1]
def load(p):
    r=json.load(open(p)); return {(c["id"],cell["sheet"],cell["cell"]):cell for c in r["cases"] for cell in c.get("cells",[])}
B,A=load(f"{S}/before_t3933.json"),load(f"{S}/after_t3933.json")
def ulpd(a,b):
    f=lambda v: (lambda i: i if i>=0 else -(i&0x7fffffffffffffff))(struct.unpack('<q',struct.pack('<d',v))[0])
    return abs(f(a)-f(b))
def vis(e,a):
    if e==a or ulpd(e,a)<=4: return False
    if e==0: return True
    E=M.floor(M.log10(abs(e))); return abs(a-e)>5*10**(E-15)
mathc={}
def mth(case,sheet,cell):
    if case not in mathc:
        su,nm=case.split("/",1); p=Path("eval/suites")/su/"cases"/nm.replace(".case.json",".math.json")
        mathc[case]=json.load(open(p))["sheets"] if p.exists() else {}
    v=mathc[case].get(sheet,{}).get(cell); return D(v["v"]) if v else None
st=collections.defaultdict(collections.Counter); ex=collections.defaultdict(list)
for k in set(B)&set(A):
    b,a=B[k],A[k]
    try: e=float(b["expected"]["v"]); xb=float(b["actual"]["v"]); xa=float(a["actual"]["v"])
    except Exception: continue
    if xb==xa: continue
    case=k[0].split("/")[1].replace(".case.json",""); s=st[case]; s["changed"]+=1
    vb,va=vis(e,xb),vis(e,xa); s["vis_b"]+=vb; s["vis_a"]+=va; s["vis_new"]+=(va and not vb)
    s["nearer_xl"]+=abs(xa-e)<abs(xb-e); s["further_xl"]+=abs(xa-e)>abs(xb-e)
    m=mth(k[0],k[1],k[2])
    if m is not None:
        fm=abs(D(xa)-m)>abs(D(xb)-m)
        s["further_math"]+=fm
        if fm and abs(xa-e)>abs(xb-e): s["further_both"]+=1; ex[case].append((k[2],b["formula"],e,xb,xa,float(m)))
for c,s in sorted(st.items()): print(c,dict(s))
for c,l in ex.items(): print(c,l[:4])
