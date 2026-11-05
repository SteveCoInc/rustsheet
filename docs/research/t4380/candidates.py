"""T-4380: score candidate UNIQUE merge rules on the two-field runs of probes/t4379 and probes/t4380 (column B).

Run from the repo root: python3 -I docs/research/t4380/candidates.py old keep pushkeep bhead pushb
Each rule changes only what the merge does when the heads of two runs are equal (`model.unique_kept` is `old`,
T-4200's rule). Results (2026-11-05, 60 runs): old 33 misses, keep 12 (the rule T-4380 chose on t4379, refuted by the
t4380 hold-out), pushkeep 5, bhead 39, pushb 37. See docs/excel-semantics/coercion.md, "No coalescing merge explains
UNIQUE's two-field runs".
"""
import json,sys,importlib.util
sys.path.insert(0,'docs/research/t4241'); import model
def kept(rows, rcmp, rule):
    n=len(rows); parent=list(range(n))
    def root(x):
        while parent[x]!=x: x=parent[x]
        return x
    def eq(a,b): return rcmp(rows[a],rows[b])==0
    def push(o,g):
        if o and (o[-1]==g or eq(o[-1],g)):
            if o[-1]!=g: parent[g]=o[-1]
        else: o.append(g)
    runs,cur=[],[]
    for g in range(n):
        if cur and rcmp(rows[g],rows[cur[-1]])<0: runs.append(cur); cur=[]
        push(cur,g)
    runs.append(cur)
    while len(runs)>1:
        nr=[]
        for j in range(0,len(runs)-1,2):
            a,b,o=list(runs[j]),list(runs[j+1]),[]
            while a and b:
                c=rcmp(rows[b[0]],rows[a[0]])
                if c==0:
                    parent[b[0]]=a[0]
                    if rule=='old': push(o,a.pop(0)); b.pop(0)
                    elif rule=='keep': b.pop(0)
                    elif rule=='pushkeep': push(o,a[0]); b.pop(0)
                    elif rule=='bhead':  # a joins b, b stays head
                        parent[b[0]]=b[0]; parent[a[0]]=b[0]; a.pop(0)
                    elif rule=='pushb':
                        parent[b[0]]=b[0]; parent[a[0]]=b[0]; push(o,b[0]); a.pop(0)
                elif c<0: push(o,b.pop(0))
                else: push(o,a.pop(0))
            for g in a+b: push(o,g)
            nr.append(o)
        if len(runs)%2: nr.append(runs[-1])
        runs=nr
    groups={}
    for i in range(n): groups.setdefault(root(i),[]).append(i)
    return sorted(min(v) for v in groups.values())
N={'­ab':'C','ab­':'B','ab':'A'}
def cases():
    out=[]
    for name,mod in [('t4379-pivotby-repeated-unreached-leaf','docs/research/t4379/gen_probe.py'),('t4380-unique-keep-head-holdout','docs/research/t4380/gen_probe.py')]:
        sp=importlib.util.spec_from_file_location('g'+name[:5],mod); g=importlib.util.module_from_spec(sp); sp.loader.exec_module(g)
        runs=[r for r,_ in g.RUNS] if hasattr(g,'RUNS') else g.runs()
        o=json.load(open(f'eval/suites/probes/cases/{name}.oracle.json'))['sheets']['Calc']
        for i,run in enumerate(runs,1):
            v=o[f'B{i}']['v'].strip('{}').split(';')
            ex=' '.join(N[p.split(',')[0].strip('"')]+p.split(',')[1].strip('"') for p in v)
            out.append((name[:5],i,run,ex))
    return out
if __name__=='__main__':
    r2=model.make_rcmp([1,1])
    for rule in sys.argv[1:]:
        miss=[]
        for nm,i,run,ex in cases():
            rows=[tuple(k) for k in run.split()]
            got=' '.join(''.join(rows[j]) for j in kept(rows,r2,rule))
            if got!=ex: miss.append((nm,i,run,ex,got))
        print(rule,'miss',len(miss)); [print('  ',m) for m in miss[:20]]
