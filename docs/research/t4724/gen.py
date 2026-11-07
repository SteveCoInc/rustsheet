"""T-4724 probe: VDB where straight line wins at period 0, in the two shapes T-4510 left unprobed.

Run: python3 -I docs/research/t4724/gen.py <case.json> <predictions.tsv>
W: a whole start with a fractional end. HS (the engine's schedule) charges share·sl first and adds n·sl;
   HP is the one product (end - start)·sl. Rows are drawn where the two differ.
F: factor 0 from a fractional start. HR (the engine, T-4111's rule) is (end - start)·(c - s)/life; HG (the fractional
   grid) is (end - start)·(c - s)/(life - f), f the start's fraction.
"""

import json
import math
import random
import sys

rng = random.Random(4724_2026_11_07)


def r15(v: float) -> float:
    return float("%.15g" % v)


def lit(v: float) -> str:
    return "%.15g" % v


w_cells: dict[str, str] = {}
f_cells: dict[str, str] = {}
out = ["sheet\trow\tformula\tH1\tH2"]
i = 0
while i < 30:
    c = r15(math.exp(rng.uniform(math.log(1e2), math.log(1e9))))
    s = r15(c * rng.uniform(0, 0.9))
    life = rng.randint(3, 40)
    factor = r15(rng.uniform(0.05, 0.95) * (1 - s / c) * life / life)
    if not (c - s) / life > c * factor / life:
        continue
    start = rng.randint(0, life - 2)
    n = rng.randint(0, life - start - 2)
    frac = r15(rng.uniform(0.1, 0.9))
    end = r15(start + n + frac)
    sl = (c - s) / life
    hs = (frac * sl) + (n * sl) if n > 0 else frac * sl
    hp = (end - start) * sl
    if hs == hp:
        continue
    i += 1
    f = f"=VDB({lit(c)},{lit(s)},{life},{start},{lit(end)},{lit(factor)})"
    w_cells[f"A{i}"] = f
    out.append(f"W\t{i}\t{f}\t{hs!r}\t{hp!r}")
for i in range(1, 21):
    c = r15(math.exp(rng.uniform(math.log(1e2), math.log(1e9))))
    s = r15(c * rng.uniform(0, 0.9))
    life = rng.randint(3, 40)
    whole = rng.randint(0, life - 2)
    frac = rng.choice([0.25, 0.5, 0.75, 0.3, 0.9])
    start = whole + frac
    end = r15(min(life, start + rng.uniform(0.5, life - start)))
    hr = (end - start) * ((c - s) / life)
    hg = (end - start) * ((c - s) / (life - frac))
    f = f"=VDB({lit(c)},{lit(s)},{life},{lit(start)},{lit(end)},0)"
    f_cells[f"A{i}"] = f
    out.append(f"F\t{i}\t{f}\t{hr!r}\t{hg!r}")
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "VDB where straight line wins at period 0, the shapes T-4510 left unprobed (T-4724); predictions in "
        "docs/research/t4724 committed before the fill. W: whole start, fractional end (H1 the schedule's "
        "share·sl + n·sl, H2 the one product (end - start)·sl). F: factor 0 from a fractional start (H1 T-4111's "
        "(end - start)·(c - s)/life, H2 the fractional grid's (end - start)·(c - s)/(life - f))."
    ),
    "tags": ["M9", "financial", "probe"],
    "sheets": [{"name": "W", "cells": w_cells}, {"name": "F", "cells": f_cells}],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
open(sys.argv[2], "w").write("\n".join(out) + "\n")
