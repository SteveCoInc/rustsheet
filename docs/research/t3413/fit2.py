"""T-3413 attempt 1: operation orders of T-2514's geometric principal sum (see fit.py)."""
from fit import cells, pmts, ulpdist
from math import log1p, expm1, exp, log
import itertools
def geoms(r,n,pv,s,e,t,p):
    L=log1p(r); out={}
    s2 = s if t==0 else max(s,2)
    m=e-s2+1
    a=-expm1(-m*L); vtail=exp(-(n-e+1)*L)
    g=1+r
    out['T2514']=(p*(a/r*g))*vtail
    out['p*a/r*g*v']=p*a/r*g*vtail
    out['p*(a*g/r)*v']=p*(a*g/r)*vtail
    out['(p*a)*(g/r)*v']=(p*a)*(g/r)*vtail
    out['p*(a/(r/g))*v']=p*(a/(r/g))*vtail
    out['p*(a/(1-1/g))*v']=p*(a/(1-1/g))*vtail
    out['p*(a/-expm1(-L))*v']=p*(a/-expm1(-L))*vtail
    out['p*a*v/(r/g)']=p*a*vtail/(r/g)
    out['p*(a/r*g*v)']=p*(a/r*g*vtail)
    out['p*a/r*(g*v)']=p*a/r*(g*vtail)
    out['p*a/r*exp(-(n-e)L)']=p*a/r*exp(-(n-e)*L)
    out['p*(a/r)*exp(-(n-e)L)']=p*(a/r)*exp(-(n-e)*L)
    out['p*a*exp(-(n-e)L)/r']=p*a*exp(-(n-e)*L)/r
    out['p/r*(exp(-(n-e)L)-exp(-(n-s2+1)L))']=p/r*(exp(-(n-e)*L)-exp(-(n-s2+1)*L))
    out['p/r*exp(-(n-e)L)*-expm1(-mL)']=p/r*exp(-(n-e)*L)*(-expm1(-m*L))
    out['p*(exp(-(n-e)L)-exp(-(n-s2+1)L))/r']=p*(exp(-(n-e)*L)-exp(-(n-s2+1)*L))/r
    out['p*exp(-(n-e)L)*-expm1(-mL)/r']=p*exp(-(n-e)*L)*(-expm1(-m*L))/r
    if t==1:
        for k in list(out): out[k+'/g']=out[k]/g
    return out
score={}
for (r,n,pv,s,e,t,X) in cells:
    m=e-s+1
    for pname,p in pmts(r,n,pv,t).items():
        for fname,x in geoms(r,n,pv,s,e,t,p).items():
            if t==1 and not fname.endswith('/g'): 
                pass
            I=-p*m - (-x)
            score.setdefault((t,pname,fname),[]).append(ulpdist(I,X))
for k,v in sorted(score.items(), key=lambda kv: (kv[0][0], -sum(1 for d in kv[1] if d==0))):
    z=sum(1 for d in v if d==0)
    if z>=4: print(z, len(v), k, v)
