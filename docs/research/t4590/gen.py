"""T-4590 probe: GROWTH and LOGEST where e^intercept or e^slope passes the double range.

Run: python3 -I docs/research/t4590/gen.py <case.json> <predictions.tsv>
"""

import json
import math
import sys

LN_MAX = math.log(sys.float_info.max)  # 709.782712893384


def r15(v: float) -> float:
    return float("%.15g" % v)


def arr(vs: list[float]) -> str:
    return "{" + ",".join("%.15g" % v for v in vs) + "}"


def fit_const(xs: list[float], ys: list[float]) -> tuple[float, float]:
    """Least squares of ln y on x with an intercept: (intercept, slope)."""
    ls = [math.log(y) for y in ys]
    n = len(xs)
    mx, ml = sum(xs) / n, sum(ls) / n
    m = sum((x - mx) * (lv - ml) for x, lv in zip(xs, ls)) / sum((x - mx) ** 2 for x in xs)
    return ml - m * mx, m


def fit_origin(xs: list[float], ys: list[float]) -> float:
    ls = [math.log(y) for y in ys]
    return sum(x * lv for x, lv in zip(xs, ls)) / sum(x * x for x in xs)


def over(v: float) -> bool:
    return v > LN_MAX


rows_int = [700, 709, 709.5, 709.7, 709.75, 709.78, 709.7827, 709.7828, 709.79, 709.8, 710, 720, 800, 1000, 1290]
rows_slope = [700, 709, 709.78, 709.79, 710, 800, 1000, 1400]
cells_int: dict[str, str] = {}
cells_slope: dict[str, str] = {}
out = ["sheet\trow\tparam\tfitted_intercept\tfitted_slope\tH1_growth\tH2_growth\tlogest_b_or_m"]
x3 = [1.0, 2.0, 3.0]
for i, c in enumerate(rows_int, 1):
    m = -c / 2
    ys = [r15(math.exp(c + m * x)) for x in x3]
    b0, m0 = fit_const(x3, ys)
    cells_int[f"A{i}"] = f"=GROWTH({arr(ys)},{arr(x3)},2,TRUE)"
    cells_int[f"B{i}"] = f"=INDEX(LOGEST({arr(ys)},{arr(x3)},TRUE),1,2)"
    cells_int[f"C{i}"] = f"=INDEX(LOGEST({arr(ys)},{arr(x3)},TRUE),1,1)"
    h = "#NUM!" if over(b0) else "number"
    out.append(f"Int\t{i}\t{c}\t{b0!r}\t{m0!r}\t{h}\t{h}\tB {h}")
xs = [0.3, 0.4, 0.5]
for i, m in enumerate(rows_slope, 1):
    ys = [r15(math.exp(m * x)) for x in xs]
    m0 = fit_origin(xs, ys)
    cells_slope[f"A{i}"] = f"=GROWTH({arr(ys)},{arr(xs)},0.4,FALSE)"
    cells_slope[f"B{i}"] = f"=INDEX(LOGEST({arr(ys)},{arr(xs)},FALSE),1,1)"
    h2 = "#NUM!" if over(m0) else "number"
    out.append(f"Slope\t{i}\t{m}\t\t{m0!r}\tnumber\t{h2}\tm {'#NUM!' if over(m0) else 'number'}")
case = {
    "format": "rustsheet-case/v1",
    "description": (
        "GROWTH and LOGEST past the double range (T-4590), predictions in docs/research/t4590/predictions.tsv "
        "committed before the fill. Int: y = e^(c - c/2 x) at x = 1, 2, 3, so the ln fit's intercept is c, from 700 "
        "to 1290 across ln(DBL_MAX) = 709.7827; A GROWTH at x = 2 (about 1), B LOGEST's b, C LOGEST's m. Slope "
        "(const FALSE): y = e^(m x) at x = 0.3, 0.4, 0.5; A GROWTH at 0.4, B LOGEST's m = e^slope. H1: GROWTH is "
        "#NUM! exactly when e^intercept overflows; H2: when e^intercept or a slope's e^coefficient overflows."
    ),
    "tags": ["M9", "statistical", "probe"],
    "sheets": [{"name": "Int", "cells": cells_int}, {"name": "Slope", "cells": cells_slope}],
    "expected": {},
}
json.dump(case, open(sys.argv[1], "w"), indent=1)
open(sys.argv[2], "w").write("\n".join(out) + "\n")
