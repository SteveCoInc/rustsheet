from lib import *
import mpmath as mp, math, json
from rustsheet_tools.fitlab.holdout import is_holdout
from rustsheet_tools.fitlab.sweep import Cell
mp.mp.prec=300
def E(a): return r53(xexp(a))
def model(x,m,s):
    d=(x-m)/s
    if not math.isfinite(d*d): return None
    v=E(-(d*d)/2)/s*K
    return 0.0 if v<2.2250738585072014e-308 else v
def exact(x,m,s): return mp.exp(-((mp.mpf(x)-m)/s)**2/2)/(s*mp.sqrt(2*mp.pi))
def cr(x,m,s):
    v=float(exact(x,m,s)); return 0.0 if v<2.2250738585072014e-308 else v
def args(c):
    a=c['args']; return (a[0],0.0,1.0) if len(a)==1 else tuple(a)
def stats(cs,f):
    b=u4=d15=0
    for c in cs:
        v=f(*args(c)); e=c['v']
        if v==e: b+=1
        if v is not None and (v==e or (e!=0 and abs(v-e)<=4*math.ulp(e))): u4+=1
        if v is not None and (v==e or (e!=0 and abs(v-e)<=abs(e)*5e-15)): d15+=1
    return b,u4,d15,len(cs)
norm=[c for c in cells if c['k']!='L']
fresh=[c for c in norm if 't0975' in c['case']]
old=[c for c in norm if 't0975' not in c['case']]
def fitlab_holdout(c):
    # The fit lab's T-3360 split: case id as `rustsheet-eval cases` names it, key `Sheet!A1`.
    sheet, address = c['ref'].split('!')
    case_id = c['case'].replace('/cases/', '/', 1)
    return is_holdout(Cell(case_id, sheet, address, c['f'], (), c['v']))
hold=[c for c in old if fitlab_holdout(c)]
train=[c for c in old if c not in hold]
for name,cs in [('train',train),('hold-out',hold),('fresh',fresh)]:
    print(name,'model',stats(cs,model),'correctly-rounded',stats(cs,cr))
# envelope on fresh: within 15 digits of Excel or 1e-9 rel of math (floor 1e-300)
out=[]
for c in fresh:
    v=model(*args(c)); e=c['v']; ex=exact(*args(c))
    ok15 = v==e or (e!=0 and abs(v-e)<=abs(e)*5e-15)
    okm = abs(mp.mpf(v)-ex)<=max(abs(ex)*1e-9,1e-300)
    if not(ok15 or okm): out.append((c['ref'],c['f'],e,v))
    if v!=e: print('fresh miss',c['ref'],c['f'],e,v)
print('envelope outside:',out)
