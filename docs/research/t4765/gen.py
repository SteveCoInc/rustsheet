"""T-4765 probe: where GROWTH is 0 because LOGEST's slopes' factor is subnormal.

Run: python3 -I docs/research/t4765/gen.py <case.json> <predictions.tsv>
Every fit is exact: y = e^(c + sum m_j x_j) at the known x, so LOGEST's ln coefficients are c and m_j.
H1: GROWTH is b · F with b = e^c and F = e^(sum m_j x_j) each a double, a subnormal one flushed to 0, the product
    flushed (so 0 once F or b is subnormal; #NUM! once b overflows; and a product past the double range #NUM!).
H2: GROWTH is e^(c + sum m_j x_j) (the engine before T-4765), #NUM! only where b overflows (T-4590).
"""

import json
import math
import sys

DBL_MIN = 2.2250738585072014e-308


def r15(v: float) -> float:
    return float("%.15g" % v)


def arr(vs: list[float]) -> str:
    return "{" + ",".join("%.15g" % v for v in vs) + "}"


def flush(v: float) -> float:
    return 0.0 if abs(v) < DBL_MIN else v


def h1(c: float, s: float) -> str:
    try:
        b = math.exp(c)
    except OverflowError:
        return "#NUM!"
    try:
        f = math.exp(s)
    except OverflowError:
        return "#NUM!"
    p = flush(b) * flush(f)
    if math.isinf(p):
        return "#NUM!"
    return repr(flush(p))


def h2(c: float, s: float) -> str:
    if c > 709.782712893384:
        return "#NUM!"
    try:
        return repr(math.exp(c + s))
    except OverflowError:
        return "#NUM!"


cells: dict[str, dict[str, str]] = {"Cut": {}, "Multi": {}, "SmallB": {}, "NoConst": {}, "Over": {}}
out = ["sheet\tcell\tformula\tH1\tH2"]


def add(sheet: str, cell: str, f: str, c: float, s: float) -> None:
    cells[sheet][cell] = f
    out.append(f"{sheet}\t{cell}\t{f}\t{h1(c, s)}\t{h2(c, s)}")


# (a) the cut: one x at 1, 2, 3; slopes' sum at the new x = 2 is -k, intercept c = k (prediction 1).
for i, k in enumerate([700, 705, 707, 708, 708.3, 708.39, 708.4, 708.5, 709, 709.5], 1):
    m = -k / 2
    ys = [r15(math.exp(k + m * x)) for x in (1.0, 2.0, 3.0)]
    add("Cut", f"A{i}", f"=GROWTH({arr(ys)},{{1,2,3}},2,TRUE)", k, -k)
# (b) two x with opposing large terms: m1·x1 = -720, m2·x2 = +720 at the new point (sum 0, prediction e^c = e^1).
pts = [(1.0, 1.0), (2.0, 1.0), (1.0, 2.0), (2.0, 3.0)]
for i, (m1, m2) in enumerate([(-360.0, 360.0), (-400.0, 400.0), (-355.0, 355.0)], 1):
    ys = [r15(math.exp(1.0 + m1 * a + m2 * b)) for a, b in pts]
    xs = "{" + ";".join(f"{a:g},{b:g}" for a, b in pts) + "}"
    add("Multi", f"A{i}", f"=GROWTH({arr(ys)},{xs},{{2,2}},TRUE)", 1.0, m1 * 2 + m2 * 2)
# (c) a subnormal b with an offsetting slope: c = -k, slope k/2 at x = 2 (prediction 1).
for i, k in enumerate([700, 708, 709, 720], 1):
    m = k / 2
    ys = [r15(math.exp(-k + m * x)) for x in (1.0, 2.0, 3.0)]
    add("SmallB", f"A{i}", f"=GROWTH({arr(ys)},{{1,2,3}},2,TRUE)", -k, k)
# (d) const FALSE with a subnormal slopes' factor at the new x: y = e^(m x) at x = 0.3, 0.4, 0.5, new x 1 or 2.
for i, (m, nx) in enumerate([(-700.0, 1.0), (-709.0, 1.0), (-360.0, 2.0), (-354.0, 2.0)], 1):
    ys = [r15(math.exp(m * x)) for x in (0.3, 0.4, 0.5)]
    add("NoConst", f"A{i}", f"=GROWTH({arr(ys)},{{0.3,0.4,0.5}},{nx:g},FALSE)", 0.0, m * nx)
# (e) the slopes' factor overflows at the new x while b keeps the prediction finite: c = -k, slope k/2, new x 2.
for i, k in enumerate([700, 710, 720], 1):
    m = k / 2
    ys = [r15(math.exp(-k + m * x)) for x in (1.0, 1.5, 2.0)]
    add("Over", f"A{i}", f"=GROWTH({arr(ys)},{{1,1.5,2}},4,TRUE)", -k, 2 * k)
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "GROWTH where LOGEST's slopes' factor or b leaves the double range (T-4765), predictions in "
        "docs/research/t4765 committed before the fill. Cut: the zero's cut in the slopes' sum (-700 to -709.5). "
        "Multi: two x with opposing large terms. SmallB: a subnormal b with an offsetting slope. NoConst: const "
        "FALSE with a subnormal factor. Over: a slopes' factor past the double range with a finite prediction."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": n, "cells": c} for n, c in cells.items()],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
open(sys.argv[2], "w").write("\n".join(out) + "\n")
