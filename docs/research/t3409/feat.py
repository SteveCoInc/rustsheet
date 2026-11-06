import json,math,time
import mpmath as mp
from statistics import NormalDist
mp.mp.dps=30
N=NormalDist()
rows=json.load(open('rows.json'))
def zq(p,tail):
    if tail=='L':
        return N.inv_cdf(p) if p<1 else float('inf')
    return -N.inv_cdf(p)
def wh(a,z):
    r=math.sqrt(a)
    c=1+0.0000886-1/(9*a)+z*0.999977894167/(3*r)+(-7.586532e-4+6.740481e-5*z*z)/a-3.08676e-3*z**3/(a*r)
    return a*c**3
def temme(A,X):
    lam=X/A
    if abs(lam-1)<mp.mpf('1e-12'):
        return mp.mpf(0.5),mp.mpf(0.5)
    e=mp.sign(lam-1)*mp.sqrt(2*(lam-1-mp.log(lam)))
    c=1/(lam-1)-1/e
    w=mp.exp(-A*e*e/2)/mp.sqrt(2*mp.pi*A)
    Q=mp.erfc(e*mp.sqrt(A/2))/2+w*c
    P=mp.erfc(-e*mp.sqrt(A/2))/2-w*c
    return P,Q
out=[]
t=time.time()
for r in rows:
    a,p,tail=r['a'],r['p'],r['tail']
    if a<1e5 or p<=0 or p>=1: continue
    try: z=zq(p,tail)
    except Exception: continue
    x0=wh(a,z)
    A=mp.mpf(a);X=mp.mpf(x0)
    lnd=float((A-1)*mp.log(X)-X-mp.loggamma(A))
    P,Q=temme(A,X)
    r.update(z=z,x0=x0,lnd=lnd,lnP0=float(mp.log(P)) if P>0 else -1e9,lnQ0=float(mp.log(Q)) if Q>0 else -1e9)
    q=float(r['mv']) if r['mv'] is not None else None
    if r['et']=='n':
        v=r['ev']
        if abs(v-x0*r['b'])<=1e-8*abs(v): kind='x0'
        elif q and abs(v-q)<=1e-10*abs(q): kind='true'
        else: kind='other'
    else: kind='err'
    r['kind']=kind
    if q: r['dq']=(x0-q/r['b'])/math.sqrt(a)
    out.append(r)
print(time.time()-t)
json.dump(out,open('feat.json','w'))
from collections import Counter
print(Counter(r['kind'] for r in out))
