import json,math,itertools
from fractions import Fraction as Fr
from eps import load
exec(open('ext.py').read())
fl=math.floor
data=load([('meta1.json','t4323-fraction-ties'),('meta2.json','t4323-fraction-sweep'),('meta3.json','t4323-fraction-sweep-2')])
T=lambda q: float(D(X(q)))
def mul(a,b,mode): return a*b if mode=='d' else T(Fr(a)*Fr(b))
def div(a,b,mode): return a/b if mode=='d' else T(Fr(a)/Fr(b))
def add(a,b,mode): return a+b if mode=='d' else T(Fr(a)+Fr(b))
CHAINS={'S':lambda s:[86400*s],'86400,s':lambda s:[86400,s],'s,86400':lambda s:[s,86400],'24,3600,s':lambda s:[24,3600,s],'24,60,60,s':lambda s:[24,60,60,s],'1440,60,s':lambda s:[1440,60,s],'24,60,60*s':lambda s:[24,60,60*s],'24,3600*s':lambda s:[24,3600*s]}
HALF=['serial_div','serial_mul','after']
res=[]
for chain,half,mode in itertools.product(CHAINS,HALF,['d','t']):
    okd0=okd=0
    for (o,c,f,s,d,k,e) in data:
        x=(d+k/86400) if d else k/86400
        S=86400*s
        if half=='serial_div': y=add(x,div(0.5,S,mode),mode)
        elif half=='serial_mul': y=add(x,mul(0.5,1/S,mode),mode)
        else: y=x
        fr=y-fl(y); P=fr
        for m in CHAINS[chain](s): P=mul(P,m,mode)
        if half=='after': P=P+0.5
        ok= f"{fl(P)%s:0{len(str(s))-1}d}"==e.rsplit('.',1)[1]
        if d==0: okd0+=ok
        else: okd+=ok
    res.append((okd0+okd,okd0,okd,chain,half,mode))
res.sort(reverse=True)
for r in res[:15]: print(r)
