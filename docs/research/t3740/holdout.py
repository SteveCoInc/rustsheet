import json,re,struct
def show(path):
    v={}
    for m in re.finditer(r'(\w+)!([A-Z]+\d+)\s+\S+\n\s+formula : [^\n]*\n\s+expected: ([^\n]*)\n\s+actual  : ([^\n]*)',open(path).read()):
        sh,c,e,a=m.groups()
        try: v[(sh,c)]=(float(e),float(a))
        except: pass
    return v
def mathv(path,sheet):
    d=json.load(open(path)); out={}
    for c,x in d['sheets'].get(sheet,{}).items():
        if isinstance(x,dict) and x.get('t')=='n': out[c]=float(x['v'])
    return out
def ulps(a,b):
    ia=struct.unpack('<q',struct.pack('<d',a))[0]; ib=struct.unpack('<q',struct.pack('<d',b))[0]; return abs(ia-ib)
def tiers(pairs):
    t=[0,0,0]
    for got,ex in pairs:
        u=ulps(got,ex); t[0]+=u==0; t[1]+=u<=4; t[2]+= '%.15g'%got=='%.15g'%ex
    return t
main=show('t3444-main.txt'); br=show('t3444-branch.txt')
m3444=mathv('wtn2/eval/suites/probes/cases/t3444-binom-range-inv-large-n.math.json','Range')
train=[f"A{r}" for r in range(6,127,6)]
print('train rows',len(train))
for name,src in (('new',br),('old',main)):
    print('train',name,tiers([(src[('Range',c)][1],src[('Range',c)][0]) for c in train]))
print('train math',tiers([(m3444[c],br[('Range',c)][0]) for c in train if c in m3444]), 'math present',sum(c in m3444 for c in train))
o=json.load(open('wtn2/eval/suites/probes/cases/t3740-binom-range-past-mean-sweep.oracle.json'))
flat={}
def walk(d,k=''):
    if isinstance(d,dict):
        if 't' in d and 'v' in d: flat[k]=d['v']; return
        for kk,vv in d.items(): walk(vv,kk)
walk(o)
mth=mathv('wtn2/eval/suites/probes/cases/t3740-binom-range-past-mean-sweep.math.json','Range')
pred=[l.split('\t') for l in open('wtn2/docs/research/t3740/predictions.tsv').read().split('\n')[1:] if l]
rows=[p for p in pred if int(p[0])<=60]
print('holdout new',tiers([(float(a),flat[f'A{r}']) for r,a,b in rows]))
print('holdout old',tiers([(float(b),flat[f'A{r}']) for r,a,b in rows]))
print('holdout math',tiers([(mth[f'A{r}'],flat[f'A{r}']) for r,a,b in rows if f'A{r}' in mth]),'present',sum(f'A{r}' in mth for r,a,b in rows))
