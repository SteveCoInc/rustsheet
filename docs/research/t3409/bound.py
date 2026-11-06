import json,re
from feat_helpers import *
d=json.load(open('feat.json'))
lines=[l for f in __import__('glob').glob('run2_*.txt') for l in open(f)]
want={}
for l in lines:
    p=l.split()
    try: n=int(p[-2])
    except: continue
    if 205<=n<=216 and p[-1]=='ok': want[p[0]]=p[1]
cells=[r for r in d if r['f'] in want]
seen=set()
for r in cells:
    if r['f'] in seen: continue
    seen.add(r['f'])
    A=mp.mpf(r['a']);x=mp.mpf(r['x0']);p=mp.mpf(r['p'])
    rr = p/(1-p) if r['tail']=='L' else (1-p)/p
    steps=[]
    for i in range(260):
        P,Q=temme(A,x);f=dens(A,x)
        s=(P*Q-rr*Q*Q)/f
        steps.append(abs(s)/x)
        x=x-s
        if abs(s)/x<1e-30: break
    def first(t): return next(i for i,v in enumerate(steps) if v<t)
    print(f"{r['kind']:5} " + ' '.join(f"{first(t):4d}" for t in (1e-6,1e-8,1e-10,1e-12,1e-14,1e-15,3e-16)) + ' ' + r['f'])
