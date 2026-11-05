import json, re, glob, sys
NUM = r'\s*(-?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s*'
PATS = {
 'S': re.compile(r'^=_?(?:xlfn\.)?NORM\.S\.DIST\('+NUM+r',\s*(FALSE|0)\s*\)$', re.I),
 'PHI': re.compile(r'^=(?:_xlfn\.)?PHI\('+NUM+r'\)$', re.I),
 'N': re.compile(r'^=(?:_xlfn\.)?NORM(?:\.)?DIST\('+NUM+','+NUM+','+NUM+r',\s*(FALSE|0)\s*\)$', re.I),
 'L': re.compile(r'^=(?:_xlfn\.)?LOGNORM\.DIST\('+NUM+','+NUM+','+NUM+r',\s*(FALSE|0)\s*\)$', re.I),
}
out=[]
for case in sorted(glob.glob('/home/user/rustsheet/eval/suites/*/cases/*.case.json')):
    orc = case.replace('.case.json','.oracle.json')
    try: o=json.load(open(orc))['sheets']
    except Exception: continue
    c=json.load(open(case))
    for s in c['sheets']:
        for ref,f in s['cells'].items():
            if not isinstance(f,str): continue
            for k,p in PATS.items():
                m=p.match(f)
                if m:
                    v=o.get(s['name'],{}).get(ref)
                    if not v or v.get('t')!='n': continue
                    args=[float(g) for g in m.groups() if g and g.upper() not in ('FALSE','0') or (g=='0' and False)]
                    # careful: numeric '0' args
                    g=m.groups()
                    nargs={'S':1,'PHI':1,'N':3,'L':3}[k]
                    args=[float(x) for x in g[:nargs]]
                    out.append(dict(case=case.split('/suites/')[1],ref=s['name']+'!'+ref,f=f,k=k,args=args,v=v['v']))
json.dump(out,open('cells.json','w'))
from collections import Counter
print(len(out), Counter(x['k'] for x in out))
