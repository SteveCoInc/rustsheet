# Hash split stated before scoring (review round 2): a cell is held out when the first hex digit of
# sha1(formula) is 0-4 (5/16, about 31%); the step limit is then chosen on the train part only.
import json,glob,hashlib
d=json.load(open('feat.json'))
B={}
for l in open('bound.txt'):
    p=l.split(); B[p[-1]]=int(p[2])
rows={}
for f in glob.glob('run2_*.txt'):
    for l in open(f):
        p=l.split()
        try:n=int(p[-2])
        except: continue
        fm=p[0]; rows[fm]=('err' if p[1]=='err' else 'val', B.get(fm, n if p[-1]=='ok' else 10**6))
held=lambda fm: hashlib.sha1(fm.encode()).hexdigest()[0] in '01234'
tr={k:v for k,v in rows.items() if not held(k)}; ho={k:v for k,v in rows.items() if held(k)}
acc=lambda S,N: sum((n<=N)==(k=='val') for k,n in S.values())
best=max(range(150,260),key=lambda N:(acc(tr,N),-abs(N-207)))
ties=[N for N in range(150,260) if acc(tr,N)==acc(tr,best)]
print('distinct formulas',len(rows),'train',len(tr),'held',len(ho))
print('train-best N',best,'ties',ties,'train',acc(tr,best),'held',acc(ho,best))
for N in (205,206,207,208,209): print(N,'train',acc(tr,N),'held',acc(ho,N))
print('always-value (correctly rounded) train',sum(k=='val' for k,n in tr.values()),'held',sum(k=='val' for k,n in ho.values()))
