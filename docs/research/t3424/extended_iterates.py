import sys, collections; sys.path.insert(0, sys.argv[1])
from score_rate import *
from fractions import Fraction as Fr
from rustsheet_tools.fitlab.exact import round_to_bits, to_double, double_round
from rustsheet_tools.fitlab.kernels.solvers import caret_pow
r64=lambda x: round_to_bits(x,64)
def resid_ext(r,A):  # residual with r extended: 1+r and /r see the extended r
    n,pmt,pv,fv,t=A
    if r==0: return Fr(pv+pmt*n+fv)
    onep=double_round(1+r)
    p=caret_pow(onep,n); k=onep if t else 1.0
    q=double_round((Fr(p)-1)/r) if True else None
    q=double_round(Fr(double_round(Fr(p)-1))/r)
    A_=double_round(Fr(k)*Fr(q)); s=double_round(Fr(double_round(Fr(pv)*Fr(p)))+Fr(double_round(Fr(pmt)*Fr(A_))))
    tot=double_round(Fr(s)+Fr(fv))
    if not math.isfinite(tot): raise ValueError
    return tot
def model_ext(a, mode):
    full=a+[0.0,0.0,0.1][len(a)-3:]
    n,pmt,pv,fv,t,g=full
    if g<=-1: return "#VALUE!"
    A=(n,pmt,pv,fv,t)
    f=(lambda r: R(to_double(r),A)) if mode=="dbl-resid" else (lambda r: resid_ext(r,A))
    def secant(g,restart):
        try:
            r0=Fr(g); f0=f(r0); r1=r64(r0-Fr(math.copysign(0.001,f0))); f1=f(r1)
            for step in range(1,201):
                flat=f1==f0
                if flat:
                    if f1==0: return "#NUM!"
                    if restart and r1!=r0 and 3<=step<=16 and 1.3e-7<=abs(f1)<1e-4 and r0>-1:
                        return secant(to_double(r0),False)
                    r2,f2=r1,f1
                else:
                    num=r64(Fr(f1)*r64(r1-r0)); den=r64(Fr(f1)-Fr(f0))
                    r2=r64(r1-r64(num/den)); f2=f(r2)
                ss=abs(r2-r1)
                if ss<Fr(1e-7) and abs(f2)<1e-7: rd=to_double(r2); return rd if 1+rd>1e-7 else "#NUM!"
                if (flat or step==200) and ss<Fr(1e-4) and abs(f2)<1e-4 and 1+r2>Fr(1e-4): return to_double(r2)
                if flat: return "#NUM!"
                r0,f0,r1,f1=r1,f1,r2,f2
            return "#NUM!"
        except Exception: return "#NUM!"
    return secant(g,True)
cs=cells()
import random
base=score(cs)
print("base exact",base[0],"type",base[1])
for mode in ("dbl-resid","ext-resid"):
    ok=typ=0
    for k,a,ev in cs:
        mv=model_ext(list(a),mode)
        ok+=mv==ev; typ+=isinstance(mv,float)==isinstance(ev,float)
    print(mode,"exact",ok,"type",typ,"of",len(cs),flush=True)
