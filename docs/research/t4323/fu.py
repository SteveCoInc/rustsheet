import math,sys
from eps import load
from frozen_fmt import fmt
fl=math.floor
def rnd(v):
    # f64::round: half away from zero, exact on the double
    r=fl(v); return r+1 if v-r>=0.5 else r
def fu(x,s):
    S=86400*s; y=x+0.5/S; F=fl((y-fl(y))*86400*s); U=rnd(x*86400*s)
    return F%s,F//s,U//s
def cur(x,s):
    U=fl(x*86400*s+0.5); return U%s,(U//s)%86400,U//s
def main():
    pairs=[tuple(a.split(':')) for a in sys.argv[1:]]
    data=load(pairs)
    for nm,m in (('half_step(FU)',fu),('current',cur)):
        r={'d0':[0,0],'d>=1':[0,0]};miss=[]
        for (o,c,f,s,d,k,e) in data:
            x=(d+k/86400) if d else k/86400
            g='d0' if d==0 else 'd>=1'; r[g][1]+=1
            p=fmt(f,s,m(x,s))
            if p==e: r[g][0]+=1
            elif d: miss.append((o[-7:],c,f,d,k,e,p))
        print(nm,r,miss[:5])
if __name__=='__main__':
    main()
