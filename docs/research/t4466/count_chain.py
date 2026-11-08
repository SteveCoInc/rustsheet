"""The count rule T-4333 adopted: for each elapsed unit, every chain of products by 24, 60, 1440, 3600 and 86400
whose product is the unit count per day, crossed with the forms of the half step. ((y*60)*60)*24 for [s], (y*60)*24 for
[m] and y*24 for [h] score every count cell (2,617 of 2,617, t4466-count-fresh included). Run from this directory;
`--train` scores the train split only (count_models.split), which picks the same chains, so the hold-out took no part
in the selection."""
import sys, math, itertools
import os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import count_models as cm
R = cm.rows([]) + cm.rows(['t4466-count-fresh'])
if '--train' in sys.argv:
    R = [r for r in R if cm.split(r[0], r[1]) == 'train']
fl=math.floor
def chains(target):
    # all orderings of factor multisets whose product is target, from primes-ish set {24,60,1440,3600,86400}
    base=[24,60,60]
    cands=set()
    for k in range(1,4):
        for combo in itertools.product([24,60,1440,3600,86400],repeat=k):
            p=1
            for c in combo: p*=c
            if p==target: cands.add(combo)
    return sorted(cands)
half={
 '0.5/(86400s)':lambda s:0.5/(86400*s),
 '0.5/s/86400':lambda s:0.5/s/86400,
 '0.5/86400/s':lambda s:0.5/86400/s,
 '1/(172800s)':lambda s:1/(172800*s),
 '0.5/s/24/60/60':lambda s:0.5/s/24/60/60,
 '0.5/s/60/60/24':lambda s:0.5/s/60/60/24,
}
units={1:chains(86400),60:chains(1440),3600:chains(24)}
best={}
for u,chs in units.items():
    sub=[r for r in R if r[3]==u]
    sc=[]
    for ch in chs:
        for hn,h in half.items():
            def m(x,s,ch=ch,h=h):
                v=x+h(s)
                for f in ch: v=v*f
                return fl(v)
            ok=sum(1 for c,cell,x,uu,s,cnt in sub if m(x,s)==cnt)
            sc.append((ok,ch,hn))
    sc.sort(reverse=True)
    print('unit',u,'cells',len(sub)); 
    for t in sc[:6]: print('  ',t)
