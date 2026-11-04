"""T-3580: build the fresh sweep probe t3580-f-fresh (seed 35800), drawn after the engine's F model was frozen
(r >= 1 arguments, bgrat at b = 1/2). Two cells per point: F.DIST.RT(x,m,n) in column A and F.DIST(x,m,n,TRUE)
in column B. Regions (mkhold.py's of T-3567, new draws; bg1 is not filtered by branch here):
  bg1   d1 = 1, n in 31..400, x = e^U(-1,4)
  d1    d1 = 1, n in 3..30, x = e^U(-8,8) n
  d2    d2 = 1, m in 1..400, x = e^U(-8,8)/m
  gen   d1, d2 log-uniform in [1, 500], 1/3 non-integer literals, x = e^U(-6,6) n/m
Every x is a decimal of at most 15 significant digits. usage: mkfresh.py [--write]"""
import json, math, random, sys
from collections import Counter
from pathlib import Path
HERE = Path(__file__).resolve().parent
CASES = HERE.parents[2] / "eval/suites/probes-numeric/cases"
NAME = "t3580-f-fresh"
rnd = random.Random(35800)
def dec(v, k=None):
    return float(f"{v:.{k or rnd.randint(3, 15)}g}")
def bg1():
    return dec(math.exp(rnd.uniform(-1, 4))), 1, rnd.randint(31, 400)
def d1():
    n = rnd.randint(3, 30)
    return dec(n * math.exp(rnd.uniform(-8, 8))), 1, n
def d2():
    m = rnd.randint(1, 400)
    return dec(math.exp(rnd.uniform(-8, 8)) / m), m, 1
def gen():
    shapes = [math.exp(rnd.uniform(0, math.log(500))) for _ in range(2)]
    shapes = [max(dec(v, 4) if rnd.random() < 1 / 3 else float(round(v)), 1.0) for v in shapes]
    ratio = math.trunc(shapes[1]) / math.trunc(shapes[0])
    return (dec(ratio * math.exp(rnd.uniform(-6, 6))), *shapes)
REGIONS = (("bg1", bg1, 400), ("d1", d1, 300), ("d2", d2, 200), ("gen", gen, 300))
pts = [(name, *draw()) for name, draw, count in REGIONS for _ in range(count)]
cells, meta = {}, []
f = lambda v: repr(v) if v != int(v) else str(int(v))
for k, (reg, x, m, n) in enumerate(pts, 1):
    cells[f"A{k}"] = f"=F.DIST.RT({f(x)},{f(m)},{f(n)})"
    cells[f"B{k}"] = f"=F.DIST({f(x)},{f(m)},{f(n)},TRUE)"
    meta.append((k, reg, x, m, n))
case = {"format": "rustsheet-case/v1",
        "description": "T-3580 fresh sweep (drawn after the freeze): F.DIST.RT and F.DIST(.,TRUE) at d1 = 1, d2 = 1 and general dfs",
        "tags": ["M9", "probe", "statistics", "T-3580", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
print(len(pts), "points", len(cells), "cells", Counter(p[0] for p in pts))
if "--write" in sys.argv:
    (CASES / f"{NAME}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (HERE / f"{NAME}.meta.json").write_text(json.dumps(meta) + "\n")
