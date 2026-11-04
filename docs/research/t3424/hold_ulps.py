import sys, collections; sys.path.insert(0, sys.argv[1])
from score_rate import *
def ulps(a,b):
    import struct
    ia=struct.unpack('<q',struct.pack('<d',a))[0]; ib=struct.unpack('<q',struct.pack('<d',b))[0]
    return ib-ia
def hold_info(a):
    full=a+[0.0,0.0,0.1][len(a)-3:]
    n,pmt,pv,fv,t,g=full; Aa=(n,pmt,pv,fv,t); f=lambda r:R(r,Aa)
    def sec(g,restart):
        r0,f0=g,f(g); r1=r0-math.copysign(0.001,f0); f1=f(r1)
        for step in range(1,201):
            if f1==f0:
                if f1==0: return None
                if restart and r1!=r0 and 3<=step<=16 and 1.3e-7<=abs(f1)<1e-4 and r0>-1: return sec(r0,False)
                if abs(f1)<1e-4 and 1+r1>1e-4: return (r0,r1,f1,f,step)
                return None
            r2=default_step(r0,f0,r1,f1); f2=f(r2)
            if abs(r2-r1)<1e-7 and abs(f2)<1e-7: return None
            r0,f0,r1,f1=r1,f1,r2,f2
        return None
    try: return sec(g,True)
    except Exception: return None
cs=cells(); C=collections.Counter(); ex=[]
for k,a,ev in cs:
    h=hold_info(list(a))
    if not h: continue
    r0,r1,f1,f,step=h
    if not isinstance(ev,float): C["excel NUM"]+=1; continue
    d=ulps(r1,ev)
    dirn=0 if r1==r0 else (1 if r1>r0 else -1)
    sgn_ex = (f(ev)>0)!=(f1>0)
    key=("same" if d==0 else ("±%d"%abs(d) if abs(d)<=4 else "far"), "zero" if dirn==0 else ("along" if d*dirn>0 else ("against" if d else "-")), "fsign flips" if sgn_ex else "")
    C[key]+=1
    if key[0]!="same" and len(ex)<12: ex.append((k,d,dirn,f1,f(ev)))
for k,v in sorted(C.items(),key=str): print(k,v)
for e in ex: print(e)
