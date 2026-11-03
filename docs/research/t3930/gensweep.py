"""T-3930 R4.2: generate probes-numeric/t3930-r4-im-small-x-cosh (seed 3930).

Written after `LIBM_COSH_BELOW` (1E-4) froze at commit edb75c46a: IMSINH, IMCOSH, IMSECH and IMCSCH of
x + yi with |x| log-uniform over 1E-6..1E-2 (densest around the cut: half the points in 3E-5..3E-4) and
|y| log-uniform over 1E-2..3, both signs.

Run: python3 docs/research/t3930/gensweep.py eval/suites/probes-numeric/cases/t3930-r4-im-small-x-cosh.case.json
"""
import json
import random
import sys

rnd = random.Random(3930)


def lit(v):
    return f"{v:.15g}".replace("e+", "E+").replace("e-", "E-")


def sign():
    return rnd.choice([-1, 1])


pts = []
for i in range(120):
    lo, hi = (-4.523, -3.523) if i % 2 == 0 else (-6, -2)
    pts.append((sign() * 10 ** rnd.uniform(lo, hi), sign() * 10 ** rnd.uniform(-2, 0.477)))
cells = {}
for i, (x, y) in enumerate(pts, start=1):
    z = f"{lit(x)}{'+' if y >= 0 else '-'}{lit(abs(y))}i"
    for col, fn in (("A", "IMSINH"), ("B", "IMCOSH"), ("C", "IMSECH"), ("D", "IMCSCH")):
        cells[f"{col}{i}"] = f'={fn}("{z}")'
case = {
    "format": "rustsheet-case/v1",
    "description": "T-3930 R4.2 fresh sweep (seed 3930, generated after LIBM_COSH_BELOW froze): IMSINH (A), IMCOSH (B), IMSECH (C), IMCSCH (D) of x+yi, |x| log-uniform 1E-6..1E-2 (every other point 3E-5..3E-4, around the 1E-4 cut), |y| log-uniform 1E-2..3, both signs.",
    "tags": ["M9", "engineering", "probe", "T-3930"],
    "sheets": [{"name": "S", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as f:
    json.dump(case, f, indent=1)
    f.write("\n")
print(len(cells))
