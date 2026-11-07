import json,glob,re,sys
pat=re.compile(r'^=RATE\(([-+0-9.Ee,]*)\)$')
out=open(sys.argv[1],'w')
n=0
for cf in sorted(glob.glob('eval/suites/*/cases/*.case.json')):
    of=cf.replace('.case.json','.oracle.json')
    try: c=json.load(open(cf))
    except Exception: continue
    try: o=json.load(open(of))
    except Exception: o=None
    for sh in c.get('sheets',[]):
        for addr,f in sh.get('cells',{}).items():
            if not isinstance(f,str): continue
            m=pat.match(f.replace(' ',''))
            if not m: continue
            key=f"{sh['name']}!{addr}"
            exp=None
            if key in c.get('expected',{}): exp=c['expected'][key]
            elif o and addr in o['sheets'].get(sh['name'],{}):
                e=o['sheets'][sh['name']][addr]; exp=e
            if exp is None: continue
            if isinstance(exp,dict):
                t=exp.get('t'); v=exp.get('v')
                ev = repr(float(v)) if t=='n' else f"E:{v}"
            else:
                ev = repr(float(exp)) if isinstance(exp,(int,float)) else f"E:{exp}"
            case=cf.split('/')[2]+'/'+cf.split('/')[-1][:-10]
            out.write(f"{case}\t{key}\t{f}\t{ev}\n"); n+=1
print(n)
