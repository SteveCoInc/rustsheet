import sys; sys.path.insert(0, sys.argv[1])
from score_rate import *
def trace(a):
    full=a+[0.0,0.0,0.1][len(a)-3:]
    n,pmt,pv,fv,t,g=full; Aa=(n,pmt,pv,fv,t); f=lambda r:R(r,Aa)
    def sec(g,restart):
        r0,f0=g,f(g); r1=r0-math.copysign(0.001,f0); f1=f(r1); fg=abs(f0); f01=abs(f1); fs=[]
        for step in range(1,201):
            flat=f1==f0
            if flat:
                if f1==0: return None
                if restart and r1!=r0 and 3<=step<=16 and 1.3e-7<=abs(f1)<1e-4 and r0>-1: return sec(r0,False)
                r2,f2=r1,f1
            else:
                r2=default_step(r0,f0,r1,f1); f2=f(r2)
            fs.append(abs(f2)); ss=abs(r2-r1)
            if ss<1e-7 and abs(f2)<1e-7: return None
            if (flat or step==200) and ss<1e-4 and abs(f2)<1e-4 and 1+r2>1e-4:
                if not flat: return None
                p=(1+r2)**n
                terms=[abs(pv*p),abs(pmt*(1+r2*t)*(p-1)/r2) if r2 else abs(pmt*n),abs(fv)]
                F=abs(f2)
                return dict(step=step,f=F,rel_max=F/max(terms),rel_pv=F/max(abs(pv),1e-300),rel_fv=F/max(abs(fv),1e-300),
                  rel_pmt=F/max(abs(pmt),1e-300),rel_guess=F/fg,rel_first=F/f01,zero=float(r1==r0),
                  ulp_ratio=F/ (2**math.frexp(max(terms))[1]*2**-53),r=abs(r2),n=n,
                  ulp_f=F/(2**math.frexp(abs(fv) if fv else 1)[1]*2**-53) )
            if flat: return None
            r0,f0,r1,f1=r1,f1,r2,f2
        return None
    try: return sec(g,True)
    except Exception: return None
cs=cells(); rows=[]
for k,a,ev in cs:
    d=trace(list(a))
    if d: rows.append((k,isinstance(ev,float),d))
print(len(rows), sum(r[1] for r in rows))
for feat in rows[0][2]:
    vals=sorted((r[2][feat],r[1]) for r in rows)
    best=None
    # threshold: below -> one class, above -> other; try both orientations
    for i in range(len(vals)+1):
        if 0<i<len(vals) and vals[i-1][0]==vals[i][0]: continue  # never split a tie
        lo=vals[:i]; hi=vals[i:]
        for below_num in (True,False):
            err=sum(1 for v,c in lo if c!=below_num)+sum(1 for v,c in hi if c==below_num)
            if best is None or err<best[0]: best=(err,i,below_num)
    err,i,bn=best
    th=vals[i][0] if i<len(vals) else None
    print(f"{feat:10} best errors {err}  at {th}  below->{'num' if bn else 'NUM!'}")
import json; 
