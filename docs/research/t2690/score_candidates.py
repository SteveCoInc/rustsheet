import json, math, re, sys

import pathlib
root=str(pathlib.Path(__file__).resolve().parents[3]/'eval/suites/probes/cases')+'/'
name=sys.argv[1] if len(sys.argv)>1 else 'linest-modes'
case=json.load(open(root+name+'.case.json')); orc=json.load(open(root+name+'.oracle.json'))
cells=case['sheets'][0]['cells']; ov=orc['sheets'][case['sheets'][0]['name']]
def rng(s):
    m=re.fullmatch(r'([A-Z])(\d+):([A-Z])(\d+)',s)
    c0,r0,c1,r1=m.group(1),int(m.group(2)),m.group(3),int(m.group(4))
    return [[cells.get(f'{chr(c)}{r}') for c in range(ord(c0),ord(c1)+1)] for r in range(r0,r1+1)]
def hh(X,y):
    n=len(y);k=len(X);A=[c[:] for c in X];b=y[:]
    for j in range(k):
        v=A[j][j:]; a0=v[0]; nrm=math.sqrt(sum(t*t for t in v))
        if nrm==0: return None
        beta_=-math.copysign(nrm,a0); tau=(beta_-a0)/beta_; sc=1.0/(a0-beta_)
        u=[1.0]+[t*sc for t in v[1:]]
        A[j][j]=beta_
        for i in range(j+1,n): A[j][i]=0.0
        for c in range(j+1,k):
            w=sum(u[i-j]*A[c][i] for i in range(j,n))*tau
            for i in range(j,n): A[c][i]-=w*u[i-j]
        w=sum(u[i-j]*b[i] for i in range(j,n))*tau
        for i in range(j,n): b[i]-=w*u[i-j]
    beta=[0.0]*k
    for j in reversed(range(k)): beta[j]=(b[j]-sum(A[c][j]*beta[c] for c in range(j+1,k)))/A[j][j]
    return beta
def H1(xs,y):  # centered
    n=len(y); mx=[sum(c)/n for c in xs]; my=sum(y)/n
    X=[[v-m for v in c] for c,m in zip(xs,mx)]; yc=[v-my for v in y]
    b=hh(X,yc); 
    if b is None: return None
    return b, my-sum(bj*m for bj,m in zip(b,mx))
def H2(xs,y):
    n=len(y); b=hh([[1.0]*n]+xs,y)
    if b is None: return None
    mx=[sum(c)/n for c in xs]; my=sum(y)/n
    return b[1:], my-sum(bj*m for bj,m in zip(b[1:],mx))
def H3(xs,y):
    n=len(y); b=hh([[1.0]*n]+xs,y)
    if b is None: return None
    return b[1:], b[0]
score={}
for addr,f in cells.items():
    if not isinstance(f,str): continue
    m=re.fullmatch(r'=LINEST\(([A-Z]\d+:[A-Z]\d+),([A-Z]\d+:[A-Z]\d+)(?:,(TRUE|))?(?:,(TRUE|FALSE|))?\)',f)
    if not m: continue
    Y=rng(m.group(1)); X=rng(m.group(2))
    try:
        y=[float(r[0]) for r in Y]; xs=[[float(r[j]) for r in X] for j in range(len(X[0]))]
    except (TypeError,ValueError): continue
    if len(y)<=len(xs)+1: continue
    col=ord(addr[0]); row=int(addr[1:]); p=len(xs)
    exp=[ov.get(f'{chr(col+i)}{row}') for i in range(p+1)]
    if any(e is None or e.get('t')!='n' for e in exp): continue
    exp=[e['v'] for e in exp]
    for nm,fn in (('H1',H1),('H2',H2),('H3',H3)):
        r=fn(xs,y)
        if r is None: continue
        got=r[0][::-1]+[r[1]]
        ok=sum(1 for a,b in zip(got,exp) if a==b)
        s=score.setdefault(nm,[0,0]); s[0]+=ok; s[1]+=len(exp)
        if nm in sys.argv[2:]: print(addr,f,nm,[ (a,b) for a,b in zip(got,exp) if a!=b])
print(score)
