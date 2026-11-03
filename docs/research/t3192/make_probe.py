"""T-3192: fresh a = b = 1/2 sweep for BETA.DIST's cdf (the hold-out of the closed-form choice).

Writes eval/suites/probes-numeric/cases/t3192-beta-half-half.case.json: x in column A (a constant), then
B =BETA.DIST(x,0.5,0.5,TRUE), C =BETADIST(x,0.5,0.5), D =2*ATAN(SQRT(x)/SQRT(1-x))/PI() (Excel's own worksheet ops).
Seeded, so rerunning reproduces the case. Usage (repo root): python3 docs/research/t3192/make_probe.py
"""
import json
import math
import random
from pathlib import Path

rng = random.Random(3192)
xs: list[float] = []
xs += [10 ** rng.uniform(-300, -5) for _ in range(60)]
xs += [rng.uniform(1e-5, 0.5) for _ in range(100)]
half = 0.5
lo, hi = half, half
xs.append(half)
for _ in range(20):
    lo, hi = math.nextafter(lo, 0), math.nextafter(hi, 1)
    xs += [lo, hi]
xs += [rng.uniform(0.5, 1) for _ in range(100)]
xs += [1 - 10 ** rng.uniform(-15, -5) for _ in range(20)]
cells: dict[str, object] = {}
for i, x in enumerate(xs, start=1):
    cells[f"A{i}"] = x
    cells[f"B{i}"] = f"=BETA.DIST(A{i},0.5,0.5,TRUE)"
    cells[f"C{i}"] = f"=BETADIST(A{i},0.5,0.5)"
    cells[f"D{i}"] = f"=2*ATAN(SQRT(A{i})/SQRT(1-A{i}))/PI()"
case = {
    "format": "rustsheet-case/v1",
    "description": "T-3192: BETA.DIST and BETADIST at a = b = 1/2 along x (fresh hold-out for the arcsine closed form), "
    "with Excel's worksheet 2*ATAN(SQRT(x)/SQRT(1-x))/PI() beside them",
    "tags": ["M9", "probe", "numeric", "T-3192", "incomplete-beta"],
    "sheets": [{"name": "Calc", "cells": cells}],
    "expected": {},
}
out = Path("eval/suites/probes-numeric/cases/t3192-beta-half-half.case.json")
out.write_text(json.dumps(case, indent=1) + "\n")
print(len(xs), "rows ->", out)
