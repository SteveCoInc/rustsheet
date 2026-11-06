import json,re,sys,glob,os
from statistics import NormalDist
R=sys.argv[1].rstrip('/')+'/eval/suites/' if len(sys.argv)>1 else 'eval/suites/'
files=[f for f in glob.glob(R+'probes/cases/*.case.json')+glob.glob(R+'probes-numeric/cases/*.case.json') if re.search(r't2982|t3070|t3350|t3396|fit-audit-t3070',f)]
pat=re.compile(r'^=(GAMMA\.INV|GAMMAINV|CHISQ\.INV\.RT|CHISQ\.INV|CHIINV)\(([^,]+),([^,]+)(?:,([^,]+))?\)$')
rows=[]
for f in files:
    case=json.load(open(f)); orc=f.replace('.case.json','.oracle.json')
    if not os.path.exists(orc): continue
    o=json.load(open(orc))['sheets']
    mp=f.replace('.case.json','.math.json'); m=json.load(open(mp))['sheets'] if os.path.exists(mp) else {}
    for sh in case['sheets']:
        for c,fm in sh['cells'].items():
            if not isinstance(fm,str): continue
            g=pat.match(fm.replace(' ',''))
            if not g: continue
            fn,p,x,y=g.groups()
            try: p=float(p); x=float(x); y=float(y) if y else None
            except: continue
            if fn in('GAMMA.INV','GAMMAINV'): a,b,tail=x,y,'L'
            elif fn=='CHISQ.INV': a,b,tail=x/2,2.0,'L'
            else: a,b,tail=x/2,2.0,'U'
            ex=o.get(sh['name'],{}).get(c)
            if ex is None: continue
            mt=m.get(sh['name'],{}).get(c,{})
            rows.append(dict(file=os.path.basename(f).split('.')[0],cell=c,f=fm,a=a,b=b,p=p,tail=tail,et=ex['t'],ev=ex.get('v'),mv=mt.get('v')))
json.dump(rows,open('rows.json','w'))
print(len(rows))
from collections import Counter
print(Counter((r['et']) for r in rows))
