"""T-3580: Gate A envelope (decision 0188 R2) on t3580-f-fresh: within 15 digits of Excel, or within 1e-9 relative
of 50-digit math (floor 1e-300). usage: gatea.py CASE ENGINE.json MATH.json (score.py and fmath.py outputs)."""
import json,math,struct,sys
from decimal import Decimal
from pathlib import Path
HERE = Path(__file__).resolve().parent
C = str(HERE.parents[2] / "eval/suites/probes-numeric/cases") + "/"
case=sys.argv[1]; eng=json.load(open(sys.argv[2]))
o=json.load(open(C+case+".oracle.json"))["sheets"]["Calc"]
mth={k:{"t":"n","v":v} for k,v in json.load(open(sys.argv[3])).items()}
meta={f"{c}{k}":reg for k,reg,*_ in json.load(open(HERE / f"{case}.meta.json")) for c in "AB"}
def ok(v):
    b=struct.unpack("<q",struct.pack("<d",v))[0]; return b if b>=0 else -(b&0x7fffffffffffffff)
inside=0; out=[]; n=0; cr=[0,0,0]
for c,e in o.items():
    n+=1
    a=eng.get(c)
    if e.get("t")!="n":
        if a is None or not a.replace('.','',1).replace('e-','').replace('-','').isdigit(): inside+=1
        else: out.append((c,"excel err",e,a))
        continue
    ev=e["v"]
    try: av=float(a)
    except: out.append((c,"engine err",ev,a)); continue
    w15 = av==ev if ev==0 else abs(av-ev)<=0.5*10**(math.floor(math.log10(abs(ev)))-14)
    m=mth.get(c)
    mok=False
    if m and m.get("t")=="n":
        mv=Decimal(m["v"]); mok = abs(Decimal(av)-mv) <= max(Decimal("1e-9")*abs(mv), Decimal("1e-300"))
        crv=float(mv); cr[0]+= crv==ev; cr[1]+= abs(ok(crv)-ok(ev))<=4
        cr[2]+= crv==ev if ev==0 else abs(crv-ev)<=0.5*10**(math.floor(math.log10(abs(ev)))-14)
    if w15 or mok: inside+=1
    else: out.append((c,meta.get(c),ev,av,m and m.get("v")))
print("cells",n,"inside",inside,"outside",len(out)); print("CR vs Excel exact/w4/w15", cr)
for x in out: print(x)
