"""T-3580: 50-digit F.DIST / F.DIST.RT of every cell of a probes-numeric case (x the double of its literal, dfs
truncated; the accuracy definition's formulas, tools/src/rustsheet_tools/accuracy/defs/fisher.py). usage: fmath.py CASE OUT.json
(run with the tools venv: cd tools && uv run python ../docs/research/t3580/fmath.py ...)."""
import json,re,sys,math
from pathlib import Path
import mpmath
from multiprocessing import Pool
mpmath.mp.dps=50
C = str(Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases") + "/"
case=sys.argv[1]; out=sys.argv[2]
cells=json.load(open(C+case+".case.json"))["sheets"][0]["cells"]
def f(item):
    k,fm=item
    mpmath.mp.dps=50
    m=re.match(r"=F\.DIST(\.RT)?\(([^,]+),([^,]+),([^,)]+)",fm)
    x=mpmath.mpf(float(m.group(2))); d1=mpmath.mpf(int(float(m.group(3)))); d2=mpmath.mpf(int(float(m.group(4))))
    if m.group(1):
        v=mpmath.mpf(1) if x==0 else mpmath.betainc(d2/2,d1/2,0,d2/(d1*x+d2),regularized=True)
    else:
        v=mpmath.mpf(0) if x==0 else mpmath.betainc(d1/2,d2/2,0,d1*x/(d1*x+d2),regularized=True)
    return k,mpmath.nstr(v,40)
with Pool() as p: r=dict(p.map(f,cells.items(),chunksize=20))
json.dump(r,open(out,"w"))
print(len(r))
