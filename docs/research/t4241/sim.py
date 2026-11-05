"""T-4241 scratch simulator: candidate GROUPBY membership models on probes/t4195-groupby-intransitive.

Run from the repo root: python3 docs/research/t4241/sim.py. It scores only the blocks whose texts are
A = ab, B = ab&U+00AD, C = U+00AD&ab (A = C, B = C, A < B), comparing each model's group memberships
(not the shown keys) with Excel's. None of the models here reaches the 42 blocks; see the task file.
"""
import json
import re

CASE = "eval/suites/probes/cases/t4195-groupby-intransitive"
NAMES = {'="ab"': "A", '="ab"&UNICHAR(173)': "B", '=UNICHAR(173)&"ab"': "C", '="AB"': "U",
         '="ab"&UNICHAR(173)&UNICHAR(173)': "E"}


def load_blocks():
    case = json.load(open(CASE + ".case.json"))
    sheets = {s["name"]: s["cells"] for s in case["sheets"]}
    data, calc = sheets["Data"], sheets["Calc"]
    oracle = json.load(open(CASE + ".oracle.json"))["sheets"]["Calc"]

    def text(cell):
        v = data.get(cell)
        return NAMES.get(v, v.strip('="')) if isinstance(v, str) else str(v)

    def name_texts(s):
        for k, v in [("ab\u00ad\u00ad", "E"), ("\u00adab", "C"), ("ab\u00ad", "B"), ('"ab"', '"A"'), ("AB", "U")]:
            s = s.replace(k, v)
        return s.replace('"', "")

    blocks = []
    for col, nf in (("C", 2), ("E", 3)):
        for r in range(1, 80):
            f = calc.get(f"{col}{r}")
            if not f:
                continue
            m = re.search(r"Data!A(\d+):[A-D](\d+)", f)
            a, b = int(m.group(1)), int(m.group(2))
            rows = [tuple(text(f"{c}{i}") for c in "ABC"[:nf]) for i in range(a, b + 1)]
            want = name_texts(str(oracle.get(f"{col}{r}", {}).get("v")))
            groups = []
            for g in want.strip("{}").split(";"):
                parts = [p.strip() for p in g.split(",")]
                groups.append((tuple(parts[:nf]), sorted(int(x) for x in parts[nf:])))
            blocks.append((col + str(r), rows, groups))
    return blocks


BLOCKS = load_blocks()
import json,re,sys,itertools
# text order: A=C, B=C, A<B ; x<y ; p<q
def tcmp(a,b):
    if a==b: return 0
    s={a,b}
    if s=={'A','C'} or s=={'B','C'}: return 0
    if s=={'A','B'}: return -1 if a=='A' else 1
    return -1 if a<b else 1
def rcmp(r,s):
    for a,b in zip(r,s):
        c=tcmp(a,b)
        if c: return c
    return 0
def natural_sort(idx,key):
    runs=[];cur=[idx[0]]
    for i in idx[1:]:
        if rcmp(key[i],key[cur[-1]])<0: runs.append(cur);cur=[i]
        else: cur.append(i)
    runs.append(cur)
    while len(runs)>1:
        nr=[]
        for j in range(0,len(runs)-1,2):
            a,b=runs[j],runs[j+1];o=[]
            while a and b:
                if rcmp(key[b[0]],key[a[0]])<0: o.append(b.pop(0))
                else: o.append(a.pop(0))
            nr.append(o+a+b)
        if len(runs)%2: nr.append(runs[-1])
        runs=nr
    return runs[0]
def lb(S,k,key):
    lo,hi=0,len(S)
    while lo<hi:
        m=(lo+hi)//2
        if rcmp(key[S[m]],k)<0: lo=m+1
        else: hi=m
    return lo
def ub(S,k,key):
    lo,hi=0,len(S)
    while lo<hi:
        m=(lo+hi)//2
        if rcmp(k,key[S[m]])<0: hi=m
        else: lo=m+1
    return lo
def first_eq(idx,key):
    kept=[]
    for i in idx:
        if not any(rcmp(key[i],key[j])==0 for j in kept): kept.append(i)
    return kept
def model_range(rows, keysel):
    n=len(rows); key={i+1:rows[i] for i in range(n)}; idx=list(range(1,n+1))
    S=natural_sort(idx,key)
    K=keysel(idx,key,S)
    res=[]
    for k in K:
        l,u=lb(S,key[k],key),ub(S,key[k],key)
        res.append(sorted(S[l:u]))
    return res
def run(name, keysel):
    ok=0;tot=0;bad=[]
    for cid,rows,groups in BLOCKS:
        if any(t in ('U','E') for r in rows for t in r): continue
        tot+=1
        got=sorted(model_range(rows,keysel)); want=sorted(g[1] for g in groups)
        if [g for g in got if g]==want: ok+=1
        else: bad.append((cid,rows,want,got))
    print(name, ok,'/',tot)
    for b in bad[:30]: print('  ',b)


def classify1(idx,vals):
    # single-field model: first-eq keys, sorted, lower-bound
    keys=[]
    for i in idx:
        if not any(tcmp(vals[i],vals[j])==0 for j in keys): keys.append(i)
    sk=natural_sort(keys,{k:(vals[k],) for k in keys})
    m={}
    for i in idx:
        lo,hi=0,len(sk)
        while lo<hi:
            mid=(lo+hi)//2
            if tcmp(vals[sk[mid]],vals[i])<0: lo=mid+1
            else: hi=mid
        m[i]=lo
    return m
def S_hier(idx,key,nf):
    # order rows by field classes hierarchically (stable)
    order=list(idx)
    def rec(group,f):
        if f==nf or len(group)<=1: return group
        m=classify1(group,{i:key[i][f] for i in group})
        out=[]
        for c in sorted(set(m.values())):
            out+=rec([i for i in group if m[i]==c],f+1)
        return out
    return rec(order,0)
def lbv(S,k,key,v):
    lo,hi=0,len(S)
    while lo<hi:
        m=(lo+hi)//2 if v==0 else (lo+hi-1)//2 if v==1 else lo+(hi-lo)//2
        if rcmp(key[S[m]],k)<0: lo=m+1
        else: hi=m
    return lo
def ubv(S,k,key,v):
    lo,hi=0,len(S)
    while lo<hi:
        m=(lo+hi)//2 if v==0 else (lo+hi-1)//2
        if rcmp(k,key[S[m]])<0: hi=m
        else: lo=m+1
    return lo
def search(Ssel,Ksel,v):
    ok=0;tot=0;bad=[]
    for cid,rows,groups in BLOCKS:
        if any(t in ('U','E') for r in rows for t in r): continue
        tot+=1
        n=len(rows); key={i+1:rows[i] for i in range(n)}; idx=list(range(1,n+1))
        S=Ssel(idx,key,len(rows[0]))
        K=Ksel(idx,key,S)
        got=sorted(sorted(S[lbv(S,key[k],key,v):ubv(S,key[k],key,v)]) for k in K)
        got=[g for g in got if g]
        want=sorted(g[1] for g in groups)
        if got==want: ok+=1
        else: bad.append((cid,rows,want,got,S,K))
    return ok,tot,bad


if __name__ == "__main__":
    S_MODELS = {"hier": S_hier, "nat": lambda idx, key, nf: natural_sort(idx, key), "input": lambda idx, key, nf: idx}
    K_MODELS = {"firsteq": lambda idx, key, S: first_eq(idx, key), "firsteqS": lambda idx, key, S: first_eq(S, key),
                "all": lambda idx, key, S: idx}
    for sn, s in S_MODELS.items():
        for kn, k in K_MODELS.items():
            for v in (0, 1):
                ok, tot, _ = search(s, k, v)
                print(sn, kn, v, ok, "/", tot)
