import math, sys
from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.kernels.solvers import fv_residual, TWICE
R = fv_residual(TWICE)
def run(args, guess=0.1, restart=True, verbose=True, depth=0):
    n,pmt,pv,fv,t = args
    a=(n,pmt,pv,fv,t)
    f=lambda r: R(r,a)
    r0,f0=guess,f(guess); r1=r0-math.copysign(0.001,f0); f1=f(r1)
    for step in range(1,201):
        flat = f1==f0
        if flat:
            if f1==0: return ("NUM-zero",step)
            if restart and r1!=r0 and 3<=step<=16 and 1.3e-7<=abs(f1)<1e-4 and r0>-1:
                if verbose: print(" "*depth+f"restart at step {step} from r0={r0!r} f1={f1:.3g}")
                return run(args,r0,False,verbose,depth+2)
            r2,f2=r1,f1
        else:
            r2=X87.add(r1,-X87.div(X87.mul(f1,X87.add(r1,-r0)),X87.add(f1,-f0)))
            f2=f(r2)
        if verbose: print(" "*depth+f"{step:3d} r={r2!r:24} f={f2: .4e} dr={abs(r2-r1):.3e}{' FLAT' if flat else ''}")
        ss=abs(r2-r1)
        if ss<1e-7 and abs(f2)<1e-7: return ("conv",r2,step)
        if (flat or step==200) and ss<1e-4 and abs(f2)<1e-4 and 1+r2>1e-4: return ("hold",r2,step)
        if flat: return ("NUM-flat",step)
        r0,f0,r1,f1=r1,f1,r2,f2
    return ("NUM-cap",)
if __name__=="__main__":
    args=tuple(float(x) for x in sys.argv[1:6]); g=float(sys.argv[6]) if len(sys.argv)>6 else 0.1
    print(run(args,g))
