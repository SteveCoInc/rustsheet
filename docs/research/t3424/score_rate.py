import json, math, re, sys
from pathlib import Path
from decimal import Decimal, ROUND_DOWN
from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.kernels.solvers import fv_residual, TWICE
R = fv_residual(TWICE)
ROOT=Path(__file__).resolve().parents[3]
def cut15(s):
    d=Decimal(s)
    if d==0: return 0.0
    e=d.adjusted(); return float(d.scaleb(-e).quantize(Decimal('1.'+'0'*14),rounding=ROUND_DOWN).scaleb(e))
NUM=r"-?\d+(?:\.\d*)?(?:[eE][-+]?\d+)?"
def arg(s):
    s=s.strip()
    m=re.fullmatch(r"\((%s)\+(%s)\)"%(NUM,NUM),s)
    if m: return X87.add(cut15(m[1]),cut15(m[2]))
    if re.fullmatch(NUM,s): return cut15(s)
    raise ValueError(s)
def cells():
    out=[]
    for suite in ("probes","probes-numeric"):
        for p in sorted((ROOT/"eval/suites"/suite/"cases").glob("*.case.json")):
            o=p.with_name(p.name.replace(".case.json",".oracle.json"))
            if not o.exists() or "RATE(" not in p.read_text(): continue
            ox=json.loads(o.read_text())["sheets"]; case=json.loads(p.read_text())
            for sh in case["sheets"]:
                for c,f in sh["cells"].items():
                    if not isinstance(f,str): continue
                    m=re.fullmatch(r"=RATE\((.*)\)",f)
                    if not m: continue
                    try: a=[arg(x) for x in m[1].split(",")]
                    except ValueError: continue
                    if not 3<=len(a)<=6: continue
                    e=ox.get(sh["name"],{}).get(c)
                    if e is None: continue
                    ev=e["v"] if e["t"]=="n" else ("#NUM!" if e.get("v")=="#NUM!" else e.get("v"))
                    out.append((f"{suite}/{p.name[:-10]}!{c}",a,ev))
    return out
def model(a, step_fn=None, flat_mode="hold", restart=True):
    full=a+[0.0,0.0,0.1][len(a)-3:]
    n,pmt,pv,fv,t,g=full
    if g<=-1: return "#VALUE!"
    A=(n,pmt,pv,fv,t)
    def f(r):
        return R(r,A)
    def secant(g,restart):
        try:
            r0,f0=g,f(g); r1=r0-math.copysign(0.001,f0); f1=f(r1)
            for step in range(1,201):
                flat=f1==f0
                if flat:
                    if f1==0: return "#NUM!"
                    if restart and r1!=r0 and 3<=step<=16 and 1.3e-7<=abs(f1)<1e-4 and r0>-1:
                        return secant(r0,False)
                    if flat_mode=="num": return "#NUM!"
                    r2,f2=r1,f1
                else:
                    r2=(step_fn or default_step)(r0,f0,r1,f1)
                    if not math.isfinite(r2): return "#NUM!"
                    f2=f(r2)
                ss=abs(r2-r1)
                if ss<1e-7 and abs(f2)<1e-7: return r2 if 1+r2>1e-7 else "#NUM!"
                if (flat or step==200) and ss<1e-4 and abs(f2)<1e-4 and 1+r2>1e-4: return r2
                if flat: return "#NUM!"
                r0,f0,r1,f1=r1,f1,r2,f2
            return "#NUM!"
        except Exception: return "#NUM!"
    return secant(g,restart)
def default_step(r0,f0,r1,f1):
    return X87.add(r1,-X87.div(X87.mul(f1,X87.add(r1,-r0)),X87.add(f1,-f0)))
def score(cs, **kw):
    ok=0; typ=0; bad=[]
    for k,a,ev in cs:
        mv=model(list(a),**kw)
        if mv==ev: ok+=1
        if isinstance(mv,float)==isinstance(ev,float): typ+=1
        else: bad.append(k)
    return ok,typ,bad
if __name__=="__main__":
    cs=cells(); print(len(cs))
    ok,typ,bad=score(cs); print("baseline exact",ok,"type",typ,len(cs)); print(bad[:40])
