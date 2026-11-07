"""T-4793 probe 2: two x where one variable's factor underflows and the sum stays normal (review round 1).

Run: python3 -I docs/research/t4793/gen2.py <case.json> <predictions.tsv>
y = e^(1 + m1 x1 + m2 x2) at (1,1), (2,1), (1,2), (2,3); known_y a column, new x (2,2).
H_flush (the engine after T-4793): 0 once any factor e^(m_j·2) is below DBL_MIN. H_sum: e^(1 + 2 m1 + 2 m2).
"""

import json
import math
import sys

DBL_MIN = 2.2250738585072014e-308


def lit(v: float) -> str:
    return "%.15g" % v


pts = [(1.0, 1.0), (2.0, 1.0), (1.0, 2.0), (2.0, 3.0)]
cells: dict[str, str] = {}
out = ["sheet\tcell\tformula\tH_flush\tH_sum"]
for i, (m1, m2) in enumerate([(-360.0, 300.0), (300.0, -360.0), (-354.5, 300.0), (-350.0, 300.0)], 1):
    ys = [float(lit(math.exp(1.0 + m1 * a + m2 * b))) for a, b in pts]
    f = ("=GROWTH({" + ";".join(lit(y) for y in ys) + "},{" + ";".join(f"{a:g},{b:g}" for a, b in pts)
         + "},{2,2},TRUE)")
    s = 1.0 + 2 * m1 + 2 * m2
    hs = repr(math.exp(s))
    hf = "0.0" if min(math.exp(2 * m1), math.exp(2 * m2)) < DBL_MIN else hs
    cells[f"A{i}"] = f
    out.append(f"One\tA{i}\t{f}\t{hf}\t{hs}")
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "GROWTH with two x where one variable's factor underflows and the sum stays normal (T-4793, review round "
        "1); predictions in docs/research/t4793/predictions2.tsv committed before the fill. H_flush: 0 once a "
        "factor e^(m_j x_j) is below DBL_MIN (the engine); H_sum: e^(sum). A4 is a normal control."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": "One", "cells": cells}],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
open(sys.argv[2], "w").write("\n".join(out) + "\n")
