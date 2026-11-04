import json, sys, math
sys.path.insert(0, sys.argv[1])
from model_trace import R, run
from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.kernels.solvers import caret_pow
ox=json.load(open(sys.argv[1]+'/../../../eval/suites/probes/cases/t3424-rate-noise-trace.oracle.json'))['sheets']['Sheet1']
meta=json.load(open(sys.argv[1]+'/probe_meta.json'))
args={}
src=open(sys.argv[1]+"/gen_trace_probe.py").read()
CELLS=eval(src[src.index("CELLS = [")+8:src.index("]\ndef iterates")+1])
A={(c,x):a for c,x,a in CELLS}
from collections import defaultdict
st=defaultdict(lambda:[0,0,0,[]])
def inner(r,a):  # model pv*P + pmt*A (twice-rounded), i.e. residual without + fv
    n,pmt,pv,fv,t=a
    p=caret_pow(X87.add(1.0,r),n); k=X87.add(1.0,r) if t else 1.0
    return X87.add(X87.mul(pv,p), X87.mul(pmt,X87.mul(k,X87.div(X87.add(p,-1.0),r))))
for cell,case,c,kind,r in meta:
    a=A[(case,c)]; e=ox[cell]
    if kind=="FV":
        m=-inner(r,a); ex=e.get('v'); s=st[(case,c)]; s[0]+=1
        if ex==m: s[1]+=1
        else: s[3].append((cell,r,ex,m,(ex-m) if isinstance(ex,float) else None))
    else:
        res=run(a,r,True,False)
        mv=res[1] if res[0] in("conv","hold") else "#NUM!"
        ev=e.get('v') if e.get('t')=='n' else e.get('v')
        st[(case,c)][2]+=0
        print("RATE",case,c,cell,"guess",repr(r),"excel",ev,"model",mv, "OK" if ev==mv else "DIFF")
for k,s in st.items():
    print(k,"FV bit-exact",s[1],"/",s[0])
    for row in s[3][:4]: print("   ",row)
