"""T-4393: score sort-then-group-neighbours UNIQUE models (no coalescing during the merge) on the 60 two-field runs of
probes/t4379 and probes/t4380 (column B). Run from the repo root: python3 -I docs/research/t4393/sort_search.py.
Sorts: insertion, binary insertion, top-down merge (both splits), bottom-up merge, natural-run merge, selection,
bubble, heap, quick (first pivot), none; source order forwards or reversed; a row joins the last group when it `=`
the group's previous row, head, any or every member; the kept row is the group's min, head, max or last index.
Also a scan with no sort (a row is kept unless it `=` a kept row). Result (2026-11-06): every model misses 35 or
more of 60 (best: selection sort, head grouping, 35), against T-4380's coalescing `pushkeep` 5.
"""
import sys, importlib.util, itertools
sys.path.insert(0,'docs/research/t4241'); import model
sp=importlib.util.spec_from_file_location('c','docs/research/t4380/candidates.py'); c=importlib.util.module_from_spec(sp); sp.loader.exec_module(c)
CASES=[(nm,i,[tuple(k) for k in run.split()],ex) for nm,i,run,ex in c.cases()]
def tc(a,b):
    if a==b: return 0
    s={a,b}
    if s=={'A','C'} or s=={'B','C'}: return 0
    return -1 if (a,b)==('A','B') else 1
def rc(r,s):
    for a,b in zip(r,s):
        x=tc(a,b) if a in 'ABC' else (a>b)-(a<b)
        if x: return x
    return 0
# sorts over index lists with cmp(i,j) on rows
def ins(idx,cmp):
    a=[]
    for i in idx:
        k=len(a)
        while k>0 and cmp(a[k-1],i)>0: k-=1
        a.insert(k,i)
    return a
def binins(idx,cmp):
    a=[]
    for i in idx:
        lo,hi=0,len(a)
        while lo<hi:
            m=(lo+hi)//2
            if cmp(i,a[m])<0: hi=m
            else: lo=m+1
        a.insert(lo,i)
    return a
def merge(a,b,cmp):
    o=[];i=j=0
    while i<len(a) and j<len(b):
        if cmp(b[j],a[i])<0: o.append(b[j]); j+=1
        else: o.append(a[i]); i+=1
    return o+a[i:]+b[j:]
def td(idx,cmp):
    if len(idx)<=1: return list(idx)
    m=len(idx)//2
    return merge(td(idx[:m],cmp),td(idx[m:],cmp),cmp)
def td2(idx,cmp):
    if len(idx)<=1: return list(idx)
    m=(len(idx)+1)//2
    return merge(td2(idx[:m],cmp),td2(idx[m:],cmp),cmp)
def bu(idx,cmp):
    runs=[[i] for i in idx]
    while len(runs)>1:
        runs=[merge(runs[k],runs[k+1],cmp) if k+1<len(runs) else runs[k] for k in range(0,len(runs),2)]
    return runs[0] if runs else []
def natural(idx,cmp):
    runs,cur=[],[]
    for i in idx:
        if cur and cmp(i,cur[-1])<0: runs.append(cur); cur=[]
        cur.append(i)
    runs.append(cur)
    while len(runs)>1:
        runs=[merge(runs[k],runs[k+1],cmp) if k+1<len(runs) else runs[k] for k in range(0,len(runs),2)]
    return runs[0]
def sel(idx,cmp):
    a=list(idx);o=[]
    while a:
        m=0
        for k in range(1,len(a)):
            if cmp(a[k],a[m])<0: m=k
        o.append(a.pop(m))
    return o
def bubble(idx,cmp):
    a=list(idx)
    for p in range(len(a)):
        for k in range(len(a)-1-p):
            if cmp(a[k],a[k+1])>0: a[k],a[k+1]=a[k+1],a[k]
    return a
def heap(idx,cmp):
    import functools
    a=list(idx);n=len(a)
    def sift(s,e):
        r=s
        while 2*r+1<=e:
            ch=2*r+1;sw=r
            if cmp(a[sw],a[ch])<0: sw=ch
            if ch+1<=e and cmp(a[sw],a[ch+1])<0: sw=ch+1
            if sw==r: return
            a[r],a[sw]=a[sw],a[r]; r=sw
    for s in range(n//2-1,-1,-1): sift(s,n-1)
    for e in range(n-1,0,-1):
        a[0],a[e]=a[e],a[0]; sift(0,e-1)
    return a
def quick(idx,cmp):
    if len(idx)<=1: return list(idx)
    p=idx[0]
    return quick([i for i in idx[1:] if cmp(i,p)<0],cmp)+[p]+quick([i for i in idx[1:] if cmp(i,p)>=0],cmp)
def nosort(idx,cmp): return list(idx)
SORTS=dict(ins=ins,binins=binins,td=td,td2=td2,bu=bu,natural=natural,sel=sel,bubble=bubble,heap=heap,quick=quick,none=nosort)
def group(order,eq,mode):
    gs=[]
    for i in order:
        if gs:
            g=gs[-1]
            ok={'prev':eq(g[-1],i),'head':eq(g[0],i),'any':any(eq(x,i) for x in g),'all':all(eq(x,i) for x in g)}[mode]
            if ok: g.append(i); continue
        gs.append([i])
    return gs
def scan(idx,eq,mode):
    # no sort: each row vs kept list
    kept=[]
    for i in idx:
        cand={'first':kept,'last':kept[-1:],'rev':kept[::-1]}[mode]
        if not any(eq(k,i) for k in cand): kept.append(i)
    return kept
res=[]
for sname,sf in SORTS.items():
  for rev in (False,True):
    for gm in ('prev','head','any','all'):
      for rep in ('min','head','max','last'):
        miss=0
        for nm,i,rows,ex in CASES:
            idx=list(range(len(rows)))
            if rev: idx=idx[::-1]
            cmp=lambda a,b: rc(rows[a],rows[b])
            eq=lambda a,b: cmp(a,b)==0
            gs=group(sf(idx,cmp),eq,gm)
            keep=sorted({'min':min,'head':lambda g:g[0],'max':max,'last':lambda g:g[-1]}[rep](g) for g in gs)
            got=' '.join(''.join(rows[j]) for j in keep)
            miss+=got!=ex
        res.append((miss,sname,rev,gm,rep))
for mode in ('first','last','rev'):
    miss=0
    for nm,i,rows,ex in CASES:
        eq=lambda a,b: rc(rows[a],rows[b])==0
        got=' '.join(''.join(rows[j]) for j in scan(list(range(len(rows))),eq,mode))
        miss+=got!=ex
    res.append((miss,'scan',mode,'',''))
res.sort(); [print(r) for r in res[:15]]
