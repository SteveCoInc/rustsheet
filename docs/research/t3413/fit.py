"""T-3413 attempt 1: score CUMPRINC principal forms through Excel's CUMIPMT = -pmt*m - CUMPRINC.

Run: python3 docs/research/t3413/fit.py (and fit2.py, which imports fit.py from its directory, for the geometric-sum orderings).
Each list is the interest's distance from Excel in ulps of the interest, per cell in `cells` order (the
T-3413 visibly-off CUMIPMT cells of fin-cumipmt-gate-edges and fin-cumipmt-tiny-rate-structure).
Python's math (glibc) stands in for Excel's exp/expm1/log1p. The `balance*` forms are type 0 only; their
type-1 rows are not a type-1 balance form and say nothing.
"""
import math, sys
from math import log1p, expm1, exp, nextafter, inf
# (rate, nper, pv, start, end, type, excel CUMIPMT)
cells = [
 (6e-5,60,1e5,3,10,0,-43.60706444761672),
 (8e-5,60,1e5,3,10,0,-58.14589024579618),
 (1e-4,60,1e5,3,10,0,-72.68628355221881),
 (1.05e-4,60,1e5,3,10,0,-76.32162670940852),
 (1.2e-4,60,1e5,3,10,0,-87.2282435768866),
 (1.5e-4,60,1e5,3,10,0,-109.0441194821833),
 (1e-4,60,1e5,5,5,0,-9.33519828870999),
 (1e-4,100,1e5,3,10,1,-75.6129877611329),
 (8e-6,100,1e5,3,10,1,-6.0480833114934285),
 (9e-6,100,1e5,3,10,1,-6.804105438500301),
 (1e-5,100,1e5,3,10,1,-7.560130167765237),
 (1.1e-5,100,1e5,3,10,1,-8.316157499096335),
 (1.2e-5,100,1e5,3,10,1,-9.072187432296232),
 (2e-5,100,1e5,3,10,1,-15.120520542116537),
 (5e-5,100,1e5,3,10,1,-37.80325097037894),
 (2e-5,100,1e5,4,4,1,-1.9400193807944106),
 (2e-5,100,1e5,6,6,1,-1.9000569694098886),
 (2e-5,100,1e5,7,7,1,-1.8800751642795603),
 (2e-5,100,1e5,10,10,1,-1.8201273510396732),
 (2e-5,100,1e5,11,11,1,-1.800143947316883),
 (2e-5,100,1e5,12,12,1,-1.7801601439258548),
 (2e-5,100,1e5,13,13,1,-1.7601759408589714),
 (2e-5,100,1e5,14,14,1,-1.7401913381080476),
 (2e-6,100,1e5,4,4,1,-0.19400019398062796),
]
def pmts(r,n,pv,t):
    L=log1p(r); d=-expm1(-n*L)
    out={}
    p0=(pv/d)*r; p1=pv*r/d
    out['(pv/d)r']=p0; out['pv r/d']=p1
    if t==1:
        out={'(pv/d)r/(1+r)':p0/(1+r),'pv r/d/(1+r)':p1/(1+r),'pv(r/(1+r))/d':pv*(r/(1+r))/d}
    return out
def princ_forms(r,n,pv,s,e,t,p):
    L=log1p(r); out={}
    m=e-s+1
    # per-period sum
    ks=range(s,e+1) if t==0 else range(max(s,2),e+1)
    off = 1 if t==0 else 2
    acc=0.0
    for k in ks: acc+= p*exp((k-off-n)*L) if t==0 else p*exp((k-1-n)*L)
    out['perperiod']=acc
    # T-2514 geometric
    mm = m if t==0 or s>1 else m-1
    g=(p*((-expm1(-mm*L))/r*(1+r)))*exp(-(n-e+1)*L)
    out['geom']=g
    # closed form
    out['closed']=p*((exp((e-n)*L)-exp((s-1-n)*L))/expm1(L))
    out['closed2']=p*exp((s-1-n)*L)*expm1(m*L)/expm1(L)
    # pv-balance drop: B(s-1)-B(e)  with B(k)=pv*(1+r)^k - p*((1+r)^k-1)/r  (type0)
    q1=exp((s-1)*L); q2=exp(e*L)
    out['balance']=(pv*q1 - p*(q1-1)/r) - (pv*q2 - p*(q2-1)/r)
    out['balance_em']=(pv*expm1((s-1)*L) - p*expm1((s-1)*L)/r) - (pv*expm1(e*L) - p*expm1(e*L)/r)
    out['balance_fact']=(pv - p/r)*(expm1((s-1)*L)-expm1(e*L))
    out['balance_fact2']=(p/r - pv)*exp((s-1)*L)*expm1(m*L)
    out['balance_fact3']=(p/r - pv)*(exp(e*L)-exp((s-1)*L))
    return out
def ulpdist(a,b):
    import struct
    ia=struct.unpack('<q',struct.pack('<d',a))[0]; ib=struct.unpack('<q',struct.pack('<d',b))[0]
    if ia<0: ia=-(ia&0x7fffffffffffffff)
    if ib<0: ib=-(ib&0x7fffffffffffffff)
    return ia-ib
score={}
for (r,n,pv,s,e,t,X) in cells:
    m=e-s+1
    for pname,p in pmts(r,n,pv,t).items():
        for iform in ['-pm-x','-(pm)-x','-p*m-x']:
            # interest = -p*m - x ; principal x sign negative
            for fname,x in princ_forms(r,n,pv,s,e,t,p).items():
                xs = -x if fname.startswith('balance') else -x
                # principal returned negative: CUMPRINC = -x
                P = -x
                if fname.startswith('balance'): P = -x  # drop of balance is positive principal paid
                I = -p*m - P
                key=(pname,fname)
                d=ulpdist(I,X)
                score.setdefault(key,[]).append(d)
            break
for k,v in sorted(score.items(), key=lambda kv: sum(1 for d in kv[1] if d==0), reverse=True):
    print(sum(1 for d in v if d==0), len(v), k, v)
