import sys, json, re, math; sys.path.insert(0, sys.argv[1])
from score_rate import arg, X87, ROOT
from rustsheet_tools.fitlab.kernels.solvers import caret_pow
A,M,D=X87.add,X87.mul,X87.div
def fv_plain(r,n,pmt,pv,t):
    if r==0: return -pv-pmt*n
    p=caret_pow(A(1.0,r),n); k=(1+r) if t else 1.0
    return -(pv*p+pmt*(k*((p-1)/r)))
def fv_twice(r,n,pmt,pv,t):
    if r==0: return -pv-pmt*n
    p=caret_pow(A(1.0,r),n); k=A(1.0,r) if t else 1.0
    return -A(M(pv,p),M(pmt,M(k,D(A(p,-1.0),r))))
res={"plain":0,"twice":0}; tot=0; diff=[]
for suite in sorted(p.name for p in (ROOT/"eval/suites").iterdir()):
    for p in sorted((ROOT/"eval/suites"/suite).rglob("*.case.json")):
        o=p.with_name(p.name.replace(".case.json",".oracle.json"))
        if not o.exists() or "FV(" not in p.read_text(): continue
        ox=json.loads(o.read_text())["sheets"]; case=json.loads(p.read_text())
        for sh in case.get("sheets",[]):
            for c,f in sh.get("cells",{}).items():
                if not isinstance(f,str): continue
                m=re.fullmatch(r"=FV\((.*)\)",f)
                if not m: continue
                try: a=[arg(x) for x in m[1].split(",")]
                except ValueError: continue
                if not 3<=len(a)<=5: continue
                a=a+[0.0,0.0][len(a)-3:]
                e=ox.get(sh["name"],{}).get(c)
                if not e or e["t"]!="n": continue
                tot+=1
                try: vp=fv_plain(*a); vt=fv_twice(*a)
                except Exception: continue
                res["plain"]+=vp==e["v"]; res["twice"]+=vt==e["v"]
                if vp!=vt: diff.append((f"{suite}/{p.name}!{c}", e["v"]==vp, e["v"]==vt))
print(tot,res); print(len(diff)); 
import collections; print(collections.Counter((x[1],x[2]) for x in diff))
for x in diff[:15]: print(x)
