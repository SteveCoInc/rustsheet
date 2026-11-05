"""T-4026: probe whether Excel's F cdf keeps or flushes a subnormal result (decision 0042 companions).

Writes eval/suites/probes/cases/t4026-f-cdf-subnormal.case.json. Column A: F.DIST(x,d1,d2,TRUE);
column B: =(A's formula)=0, which tells Excel's true 0 from a kept subnormal (Graph shows both as 0).
Rows 1-6: the six Tier A cells as filed. Then three families walked in ln(cdf) from -700 to -716 by 0.5.
Note: F truncates its dfs, so the (377, 1.123) family is (377, 1) in Excel and, solved for 1.123, never reaches the
edge (cdf 1.1E-295 to 1.3E-302); the two other families carry the finding.
Run: cd tools && uv run python ../docs/research/t4026/mkprobe.py
"""

import json
from pathlib import Path

import mpmath as mp

mp.mp.dps = 60
TIER_A = [
    ("3.66425083e-05", 316, "1"),
    ("7.78e-06", 225, "1"),
    ("4.7981983151322e-06", 207, "1"),
    ("2.91738108e-06", 191, "1"),
    ("4.37662e-05", 337, "1"),
    ("6.11179254226e-05", 377, "1.123"),
]
FAMILIES = [(316, "1"), (40, "1"), (377, "1.123")]


def ln_cdf(x, d1, d2):
    d1, d2 = mp.mpf(d1), mp.mpf(d2)
    z = d1 * x / (d1 * x + d2)
    return mp.log(mp.betainc(d1 / 2, d2 / 2, 0, z, regularized=True))


def solve(target, d1, d2):
    f = lambda lx: ln_cdf(mp.e**lx, d1, d2) - target  # noqa: E731
    lo, hi = mp.mpf(-200), mp.mpf(5)
    for _ in range(200):
        mid = (lo + hi) / 2
        if f(mid) > 0:
            hi = mid
        else:
            lo = mid
    return mp.e**lo


def main():
    rows = [(x, d1, d2) for x, d1, d2 in TIER_A]
    for d1, d2 in FAMILIES:
        t = -700.0
        while t >= -716.0:
            x = solve(t, d1, d2)
            rows.append((mp.nstr(x, 12, strip_zeros=True).replace("e", "E"), d1, d2))
            t -= 0.5
    cells = {}
    for i, (x, d1, d2) in enumerate(rows, 1):
        f = f"F.DIST({x},{d1},{d2},TRUE)"
        cells[f"A{i}"] = "=" + f
        cells[f"B{i}"] = f"=({f})=0"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4026: does F's cdf keep or flush a subnormal result? Column B is the decision 0042 companion "
            "(=(A)=0). Rows 1-6 are T-4026's Tier A cells; rows 7-39 (d1 316, d2 1), 40-72 (40, 1) and "
            "73-105 (377, 1.123) walk ln of the true cdf from -700 to -716 by 0.5 (60-digit mpmath)"
        ),
        "tags": ["M9", "probe", "statistics", "T-4026"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    out = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4026-f-cdf-subnormal.case.json"
    out.write_text(json.dumps(case, indent=2) + "\n")
    print(out, len(rows))


main()
