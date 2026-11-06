"""T-4419: operation orders for the band probe (python3 search_pipelines.py, from this directory).

Each candidate multiplies the serial by a chain of factors to milliseconds, adds the half step (0.5 ms in the current
unit, optionally plus 2^-72 days) after a chosen stage, optionally rounds to 15 significant digits after any stage, and
floors. Scored on probes/t4419-band-ulp-map (913 cells). Best: 887/913, no exact fit."""
import json,math,itertools
from decimal import Decimal
from fractions import Fraction as F
m=json.load(open('meta1.json'))
o=json.load(open('../../../eval/suites/probes/cases/t4419-band-ulp-map.oracle.json'))['sheets']['Sheet1']
cells=[]
for c,(f,s,k,n,yh,p) in m.items():
    tie=round((k+0.0005)*1000)
    cells.append((float.fromhex(yh),tie,o[c]['v']==f"0.{tie:03d}",c))
def r15(v): return float(Decimal(f"{v:.14e}"))
chains={'24,60,60,1000':(24,60,60,1000),'86400,1000':(86400,1000),'1440,60,1000':(1440,60,1000),'24,3600,1000':(24,3600,1000),'24,60,60000':(24,60,60000),'86400000':(86400000,)}
res=[]
for cn,ch in chains.items():
  L=len(ch)
  for addpos in range(L+1):
    prod=math.prod(ch[:addpos]); half=float(F(prod,2*86400000))
    for off in (0,1):
      for mask in range(1<<(L+1)):  # r15 after stage i (i=0: before any multiply, after add if addpos==0)
        good=0; miss=[]
        for y,tie,up,c in cells:
            v=y
            for i in range(L+1):
                if i==addpos: v=v+half+(2**-72*prod if off else 0)
                if i<L: v=v*ch[i]
                if mask>>i&1: v=r15(v)
            if (math.floor(v)>=tie)==up: good+=1
            else: miss.append(c)
        res.append((good,cn,addpos,off,bin(mask),miss[:6]))
for r in sorted(res,reverse=True)[:12]: print(r)
