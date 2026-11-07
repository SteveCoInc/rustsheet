"""T-4793 probe: GROWTH with several x at the factor edges, and a subnormal product of normal factors.

Run: python3 -I docs/research/t4793/gen.py <case.json> <predictions.tsv>
Multi: y = e^(1 + m1 x1 + m2 x2) at (1,1), (2,1), (1,2), (2,3), known_y a column against the 4x2 known_x, new x (2,2).
  H1 (the engine): one summed slopes' factor F = e^(m1·2 + m2·2); H3: one factor per variable, e^(m_j·2) each.
SubProd: y = e^(c + m x) at x = 0.5, 1, 1.5, new x 2, with b = e^c and F = e^(2m) normal and their product near or
  below DBL_MIN. H1 (the engine): e^(c + 2m) as it is; H4: the product flushed to 0 once subnormal.
"""

import json
import math
import sys

DBL_MIN = 2.2250738585072014e-308


def r15(v: float) -> float:
    return float("%.15g" % v)


def lit(v: float) -> str:
    return "%.15g" % v


def exp_or(v: float) -> str:
    try:
        r = math.exp(v)
    except OverflowError:
        return "#NUM!"
    return repr(r)


cells: dict[str, dict[str, str]] = {"Multi": {}, "SubProd": {}}
out = ["sheet\tcell\tformula\tH1\tH_alt"]
pts = [(1.0, 1.0), (2.0, 1.0), (1.0, 2.0), (2.0, 3.0)]
for i, (m1, m2) in enumerate([(-360.0, 360.0), (-400.0, 400.0), (-355.0, 355.0), (-100.0, 100.0)], 1):
    ys = [r15(math.exp(1.0 + m1 * a + m2 * b)) for a, b in pts]
    ycol = "{" + ";".join(lit(y) for y in ys) + "}"
    xs = "{" + ";".join(f"{a:g},{b:g}" for a, b in pts) + "}"
    f = f"=GROWTH({ycol},{xs},{{2,2}},TRUE)"
    h1 = exp_or(1.0 + 2 * m1 + 2 * m2)
    f1, f2 = 2 * m1, 2 * m2
    if f1 > 709.782712893384 or f2 > 709.782712893384:
        h3 = "#NUM!"
    elif min(f1, f2) < math.log(DBL_MIN):
        h3 = "0.0"
    else:
        h3 = repr(math.e * math.exp(f1) * math.exp(f2))
    cells["Multi"][f"A{i}"] = f
    out.append(f"Multi\tA{i}\t{f}\t{h1}\t{h3}")
for i, (c, m) in enumerate([(-400.0, -154.0), (-400.0, -154.5), (-400.0, -155.0), (-400.0, -160.0),
                            (-350.0, -179.0), (-300.0, -205.0)], 1):
    ys = [r15(math.exp(c + m * x)) for x in (0.5, 1.0, 1.5)]
    f = f"=GROWTH({{{','.join(lit(y) for y in ys)}}},{{0.5,1,1.5}},2,TRUE)"
    s = c + 2 * m
    h1 = repr(math.exp(s))
    h4 = "0.0" if math.exp(s) < DBL_MIN else h1
    cells["SubProd"][f"A{i}"] = f
    out.append(f"SubProd\tA{i}\t{f}\t{h1}\t{h4}")
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "GROWTH with several x at the factor edges and a subnormal product of normal factors (T-4793), predictions "
        "in docs/research/t4793 committed before the fill. Multi: known_y a column against a 4x2 known_x, opposing "
        "slope terms (H1 summed factor, H3 one per variable). SubProd: b and F normal, b·F near or below DBL_MIN "
        "(H1 e^(sum) as it is, H4 flushed to 0)."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": n, "cells": c} for n, c in cells.items()],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
open(sys.argv[2], "w").write("\n".join(out) + "\n")
