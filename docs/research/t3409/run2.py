import json,math,sys
from feat_helpers import *
d=json.load(open('feat.json'))
lo=float(sys.argv[1]); hi=float(sys.argv[2])
sel=[r for r in d if r['lnd']>-708.39 and lo<=r['a']<hi and r['kind']!='x0' and not (r['f'].startswith('=CHISQ.INV(') and r['a']>5e9)]
out=[]
for r in sel:
    n,st,x=odds_newton(r['a'],r['x0'],r['p'],r['tail'])
    xv=float(x)*r['b']; ev=r['ev'] if r['kind']!='err' else None
    out.append(dict(file=r['file'],cell=r['cell'],f=r['f'],a=r['a'],z=r['z'],kind=r['kind'],n=n,st=st,zs=(r['x0']-r['a'])/math.sqrt(r['a'])))
    print(r['f'],r['kind'],(abs(xv-ev)/ev if ev else ''),n,st,flush=True)
json.dump(out,open(f'run2_{sys.argv[1]}.json','w'))
