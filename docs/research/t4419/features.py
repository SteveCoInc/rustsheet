"""Fit the band-dense threshold windows (after dense.py) against sawtooth features: the rounding error of doubles built
from k and of the products at the tie. A constant fits 100 of 164 ties; no single feature adds more than one."""
import os
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'dense_res.json')
import json, itertools
from fractions import Fraction as F
res=json.load(open(OUT))
ulp=F(2)**-71
def err(q): return F(float(q))-q
W=[]
for k,(sw,mono,lo,hi) in res.items():
    k=int(k); xt=F(2*k+1,2)/86400000; xd=F(float(xt)); off=(xd-xt)/ulp
    thr=float(off+sw-F(1,2))
    feats={
     'tie_s fl((k+.5)/1000)': float(err(F(2*k+1,2000))/86400/ulp),
     'fl(k/1000)': float(err(F(k,1000))/86400/ulp),
     'fl((k+1)/1000)': float(err(F(k+1,1000))/86400/ulp),
     'fl(k.5) /86400000 off': float(off),
     'prod xd*86400': float((F(float(xd*86400))-xd*86400)/86400/ulp),
     'prod xd*86400000': float((F(float(xd*86400000))-xd*86400000)/86400000/ulp),
     'prod (xd*86400)*1000': float((F(float(F(float(xd*86400))*1000))-xd*86400000)/86400000/ulp),
     'prod xd*24': float((F(float(xd*24))-xd*24)/24/ulp),
     'chain24.60.60': float((F(((float(xd)*24.0)*60.0)*60.0)-xd*86400)/86400/ulp),
     'chain24.60.60.1000': float((F((((float(xd)*24.0)*60.0)*60.0)*1000.0)-xd*86400000)/86400000/ulp),
     'fl(1/24)-ish x/(1/24)': float((F(float(xd)/(1/24))-xd*24)/24/ulp),
    }
    W.append((k,thr-0.5,thr+0.5,feats))
names=list(W[0][3])
def best_fit(fs):
    # search coefficients on grid, constant via interval intersection count
    best=(0,None)
    grid=[i/4 for i in range(-12,13)]
    for coefs in itertools.product(grid,repeat=len(fs)):
        # threshold T = c + sum a*f ; need lo<=T<hi -> c in [lo - s, hi - s)
        iv=[(lo-sum(a*f[n] for a,n in zip(coefs,fs)), hi-sum(a*f[n] for a,n in zip(coefs,fs))) for k,lo,hi,f in W]
        pts=sorted(set([a for a,b in iv]))
        for c in pts:
            cnt=sum(1 for a,b in iv if a<=c<b)
            if cnt>best[0]: best=(cnt,(coefs,c))
    return best
for n in names: print(len(W), n, best_fit([n]))
