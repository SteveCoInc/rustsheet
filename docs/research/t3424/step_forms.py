import sys; sys.path.insert(0, sys.argv[1])
from score_rate import *
A,M,D=X87.add,X87.mul,X87.div
V={
 "default": None,
 "plain": lambda r0,f0,r1,f1: r1 - f1*(r1-r0)/(f1-f0),
 "from_r0": lambda r0,f0,r1,f1: A(r0,-D(M(f0,A(r1,-r0)),A(f1,-f0))),
 "slope_first": lambda r0,f0,r1,f1: A(r1,-D(f1,D(A(f1,-f0),A(r1,-r0)))),
 "ratio_first": lambda r0,f0,r1,f1: A(r1,-M(A(r1,-r0),D(f1,A(f1,-f0)))),
 "div_then_mul": lambda r0,f0,r1,f1: A(r1,-M(D(A(r1,-r0),A(f1,-f0)),f1)),
 "weighted": lambda r0,f0,r1,f1: D(A(M(r0,f1),-M(r1,f0)),A(f1,-f0)),
}
cs=cells()
base=set(score(cs)[2])
for name,fn in V.items():
    for fm in ("hold","num"):
        ok,typ,bad=score(cs,step_fn=fn,flat_mode=fm)
        print(f"{name:14} flat={fm:4} exact={ok} type={typ} wrongtype={len(bad)} fixed={len(base-set(bad))} broke={len(set(bad)-base)}",flush=True)
