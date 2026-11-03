"""T-3795 R4.2: generate probes-numeric/t3795-r4-im-last-digits (seed 3795).

Written after the T-3795 models froze (commit 584b6fb63): IMSQRT's twice-rounded double root,
IMLOG10 and IMLOG2 as IMLN's parts times one double constant with Excel's ATAN2 kernel as the
argument, and IMSECH's unfused `sinh²x + cos²y`.

Run: python3 docs/research/t3795/gensweep.py eval/suites/probes-numeric/cases/t3795-r4-im-last-digits.case.json
"""
import json
import math
import random
import sys

rnd = random.Random(3795)


def lit(v):
    return f"{v:.15g}".replace("e+", "E+").replace("e-", "E-")


def sign():
    return rnd.choice([-1, 1])


wide = []  # IMSQRT, IMLOG10, IMLOG2: log-uniform over both parts, both signs
for _ in range(80):
    wide.append((sign() * 10 ** rnd.uniform(-150, 150), sign() * 10 ** rnd.uniform(-150, 150)))
for _ in range(50):  # |z| within ~1e-13 of 1, any angle
    t = rnd.uniform(-math.pi, math.pi)
    d = sign() * 10 ** rnd.uniform(-16, -13)
    wide.append(((1 + d) * math.cos(t), (1 + d) * math.sin(t)))
for _ in range(20):  # one part a few ulps from +-1, the other 0 or small
    a = sign() * (1 + sign() * 10 ** rnd.uniform(-15.5, -14.5))
    b = rnd.choice([0.0, sign() * 10 ** rnd.uniform(-300, -6)])
    wide.append((a, b) if rnd.random() < 0.5 else (b, a))
sech = []  # IMSECH: the finite domain (|x| up to 30, |y| up to 1000)
for _ in range(100):
    sech.append((sign() * 10 ** rnd.uniform(-3, 1.5), sign() * 10 ** rnd.uniform(-3, 3)))
for _ in range(30):  # x a few ulps from +-1
    sech.append((sign() * (1 + sign() * 10 ** rnd.uniform(-15.5, -14.5)), sign() * 10 ** rnd.uniform(-2, 2)))

cells = {}
for i, (x, y) in enumerate(wide, start=1):
    for col, fn in (("A", "IMSQRT"), ("B", "IMLOG10"), ("C", "IMLOG2")):
        cells[f"{col}{i}"] = f"={fn}(COMPLEX({lit(x)},{lit(y)}))"
for i, (x, y) in enumerate(sech, start=1):
    cells[f"D{i}"] = f"=IMSECH(COMPLEX({lit(x)},{lit(y)}))"
case = {
    "format": "rustsheet-case/v1",
    "description": "T-3795 R4.2 fresh sweep (seed 3795, generated after the T-3795 models froze): IMSQRT (A), IMLOG10 (B), IMLOG2 (C) of COMPLEX(x, y) for 80 log-uniform points over both parts (1E-150..1E150, both signs), 50 with |z| within 1E-13 of 1 and 20 with one part a few ulps from ±1; IMSECH (D) for 100 points with |x| 1E-3..31.6 and |y| 1E-3..1000 and 30 with x a few ulps from ±1.",
    "tags": ["M9", "engineering", "probe", "T-3795"],
    "sheets": [{"name": "S", "cells": cells}],
    "expected": {},
}
with open(sys.argv[1], "w") as f:
    json.dump(case, f, indent=1)
    f.write("\n")
print(len(cells))
