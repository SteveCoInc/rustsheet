"""T-3192: second fresh a = b = 1/2 sweep, the out-of-sample test of the switch frozen after the first
(`score_half.py`): the direct form 2*atan(sqrt(x)/sqrt(1-x))/pi up to x = 0.9, the reflected form above.

Writes eval/suites/probes-numeric/cases/t3192-beta-half-half-2.case.json: x in column A, B =BETA.DIST(x,0.5,0.5,TRUE).
Seeded (a different seed from make_probe.py). Usage (repo root): python3 docs/research/t3192/make_probe2.py
"""
import json
import math
import random
from pathlib import Path

rng = random.Random(31922)
xs: list[float] = []
xs += [rng.uniform(0.0, 0.8) for _ in range(80)]
xs += [rng.uniform(0.8, 1.0) for _ in range(150)]
xs += [rng.uniform(0.88, 0.92) for _ in range(60)]
xs += [1 - 10 ** rng.uniform(-15, -1) for _ in range(30)]
x = 0.9
xs.append(x)
lo, hi = x, x
for _ in range(10):
    lo, hi = math.nextafter(lo, 0), math.nextafter(hi, 1)
    xs += [lo, hi]
cells: dict[str, object] = {}
for i, v in enumerate(xs, start=1):
    cells[f"A{i}"] = v
    cells[f"B{i}"] = f"=BETA.DIST(A{i},0.5,0.5,TRUE)"
case = {
    "format": "rustsheet-case/v1",
    "description": "T-3192: BETA.DIST at a = b = 1/2 along x, second fresh sweep (out-of-sample test of the x = 0.9 "
    "switch between the direct and the reflected arctangent form)",
    "tags": ["M9", "probe", "numeric", "T-3192", "incomplete-beta"],
    "sheets": [{"name": "Calc", "cells": cells}],
    "expected": {},
}
out = Path("eval/suites/probes-numeric/cases/t3192-beta-half-half-2.case.json")
out.write_text(json.dumps(case, indent=1) + "\n")
print(len(xs), "rows ->", out)
