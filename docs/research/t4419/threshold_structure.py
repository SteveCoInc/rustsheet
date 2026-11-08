"""Structure of the band-dense switch thresholds (after dense.py): the threshold in the exact-tie frame, its regression
on the tie double's offset (slope about -1.23), and the segments one constant threshold covers."""
import os
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'dense_res.json')
import json
from fractions import Fraction as F
res=json.load(open(OUT))
ulp=F(2)**-71
rows=[]
for k,(sw,mono,lo,hi) in res.items():
    k=int(k); xt=F(2*k+1,2)/86400000; xd=F((k+0.5)/86400000)
    off=(xd-xt)/ulp                     # double tie vs exact tie, in x-ulps
    thr_x = off + sw - 0.5              # threshold in x-ulps from the exact tie (midpoint between sw-1 and sw)
    rows.append((k,sw,float(off),float(thr_x)))
# threshold in exact x-ulps relative to exact tie: print a compact listing and stats
import statistics
th=[r[3] for r in rows]
print('threshold x-ulps from exact tie: mean %.3f sd %.3f min %.3f max %.3f'%(statistics.mean(th),statistics.pstdev(th),min(th),max(th)))
# convert threshold to seconds units 5e-16
s_unit=float(ulp*86400/F(5,10**16))
print('1 x-ulp = %.4f units of 5e-16 s'%s_unit)
# correlation with off
import math
def corr(a,b):
    ma,mb=statistics.mean(a),statistics.mean(b); 
    return sum((x-ma)*(y-mb) for x,y in zip(a,b))/math.sqrt(sum((x-ma)**2 for x in a)*sum((y-mb)**2 for y in b))
print('corr(sw, off)=%.3f'%corr([r[1] for r in rows],[r[2] for r in rows]))
print('corr(thr, k)=%.3f'%corr([r[3] for r in rows],[r[0] for r in rows]))
# threshold in seconds from exact tie: thr_x*ulp*86400 ; in units of the double s-ulp at that k
for lo_k,hi_k in [(165,249),(250,256),(256,329)]:
    sub=[r for r in rows if lo_k<=r[0]<hi_k]
    print(lo_k,hi_k,'mean thr %.3f sd %.3f'%(statistics.mean([r[3] for r in sub]),statistics.pstdev([r[3] for r in sub])))
xs=[r[2] for r in rows]; ys=[r[1] for r in rows]
mx,my=statistics.mean(xs),statistics.mean(ys)
b=sum((x-mx)*(y-my) for x,y in zip(xs,ys))/sum((x-mx)**2 for x in xs); a=my-b*mx
res=[y-(a+b*x) for x,y in zip(xs,ys)]
print('sw = %.3f + %.3f*off ; resid sd %.3f'%(a,b,statistics.pstdev(res)))
# residual vs k structure: print residual sign pattern by k
print(''.join('+' if r>0.5 else ('-' if r<-0.5 else '.') for r in res))
print('--- thr (exact frame, x-ulps) by k, as bins of 0.5 ulp')
for r in rows:
    k,sw,off,thr=r
    bar=int(round((thr+16)*4))
    if k%1==0: print(f"{k} {thr:7.2f} "+' '*bar+'*')
print('--- segments with one constant threshold (exact frame, x-ulps)')
segs=[]; cur=None
for k,sw,off,thr in rows:
    lo,hi=thr-0.5,thr+0.5
    if cur and max(cur[2],lo)<min(cur[3],hi):
        cur=[cur[0],k,max(cur[2],lo),min(cur[3],hi)]
    else:
        if cur: segs.append(cur)
        cur=[k,k,lo,hi]
segs.append(cur)
for s in segs: print('k %d-%d  T in [%.2f, %.2f)'%tuple(s))
