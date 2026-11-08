"""Read probes/t4419-band-dense and write each tie's switch point and threshold window to dense_res.json (run from the
repository root: python3 docs/research/t4419/dense.py). Prints k:switch for every tie; a "!" would mark a non-monotone tie
(none is)."""
import os
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'dense_res.json')
import json,re
from fractions import Fraction as F
c=json.load(open('eval/suites/probes/cases/t4419-band-dense.case.json'))['sheets'][0]['cells']
o=json.load(open('eval/suites/probes/cases/t4419-band-dense.oracle.json'))['sheets']['Sheet1']
pat=re.compile(r'=TEXT\((\d+)\.5/86400000\+\((-?\d+)\)\*2\^\(-71\),"s.000"\)')
by={}
for k,f in c.items():
    kk,n=map(int,pat.match(f).groups()); v=o[k]['v']; up=int(v.split('.')[1])==kk+1
    x=(kk+0.5)/86400000+n*2.0**-71
    by.setdefault(kk,[]).append((n,up,x))
U=F(5,10**16)
res={}
for kk,r in sorted(by.items()):
    r.sort(); ups=[n for n,u,_ in r if u]; sw=min(ups) if ups else None
    mono = sw is None or all((n>=sw)==u for n,u,_ in r)
    t=F(2*kk+1,2000)
    pos={n:float((F(x)*86400-t)/U) for n,u,x in r}
    lo = -1-pos[sw] if sw is not None and sw>-16 else None
    hi = -1-pos[sw-1] if sw is not None and sw>-16 else None
    res[kk]=(sw,mono,lo,hi)
json.dump({k:v for k,v in res.items()},open(OUT,'w'))
line=''
for kk,(sw,mono,lo,hi) in res.items():
    line+=f"{kk}:{sw}{'' if mono else '!'} "
print(line)
