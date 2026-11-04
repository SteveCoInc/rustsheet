"""T-3754 grid: the engine's F tails against 50-digit math at a small d1, a huge d2 and x near 1.

    python grid.py generate ROOT          # write ROOT/eval/suites/t3754/{suite.toml,cases/grid.case.json}
    rustsheet-eval show t3754/grid.case.json --all --root ROOT > SHOW.txt
    python grid.py score SHOW.txt         # max relative error per (d1 band, d2 decade) stratum

The case's expected values are the math (mpmath 1.3, mp.dps = 50): F.DIST(x,d1,d2,TRUE) = I_u(d1/2, d2/2) and
F.DIST.RT(x,d1,d2) = I_v(d2/2, d1/2), u = d1 x/(d1 x + d2), v = d2/(d1 x + d2), on the doubles the literals parse to.
No oracle is involved: this is an accuracy measurement (decision 0114 `accuracy` stage).
"""

from __future__ import annotations

import json
import os
import re
import sys
from collections import defaultdict

import mpmath

D1 = [1, 2, 3, 4, 5, 6, 7, 8, 10, 13, 16, 20]
D2 = [1e5, 3e5, 1e6, 3e6, 1e7, 1.5e7, 2e7, 2.5e7, 3e7, 1e8, 3e8, 1e9]
X = [0.5, 0.75, 1.0, 1.09080885395379, 1.25, 1.5, 2.0]


def lit(v: float) -> str:
    return f"{v:.15g}".replace("e+", "E+").replace("e-", "E-")


def points():
    for d1 in D1:
        for d2 in D2:
            for x in X:
                yield d1, d2, x


def generate(root: str) -> None:
    mpmath.mp.dps = 50
    cells, expected, meta = {}, {}, {}
    row = 1
    for d1, d2, x in points():
        md1, md2, mx = mpmath.mpf(d1), mpmath.mpf(d2), mpmath.mpf(float(lit(x)))
        u = md1 * mx / (md1 * mx + md2)
        v = md2 / (md1 * mx + md2)
        cdf = mpmath.betainc(md1 / 2, md2 / 2, 0, u, regularized=True)
        sf = mpmath.betainc(md2 / 2, md1 / 2, 0, v, regularized=True)
        args = f"{lit(x)},{d1},{lit(d2)}"
        cells[f"A{row}"] = f"=F.DIST({args},TRUE)"
        cells[f"B{row}"] = f"=F.DIST.RT({args})"
        expected[f"S!A{row}"] = {"t": "n", "v": float(cdf)}
        expected[f"S!B{row}"] = {"t": "n", "v": float(sf)}
        row += 1
    suite = os.path.join(root, "eval", "suites", "t3754")
    os.makedirs(os.path.join(suite, "cases"), exist_ok=True)
    with open(os.path.join(suite, "suite.toml"), "w") as f:
        f.write('name = "t3754"\ndescription = "T-3754 grid"\nkind = "spec"\nroot = "cases"\n'
                'include = ["**/*.case.json"]\nlicense = "Apache-2.0"\norigin = "generated"\ngate = false\n'
                'baseline = "cells"\nmilestone = ["M9"]\n')
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-3754 grid (math expectations)",
        "tags": ["M9", "statistics", "T-3754"],
        "sheets": [{"name": "S", "cells": cells}],
        "expected": expected,
    }
    with open(os.path.join(suite, "cases", "grid.case.json"), "w") as f:
        json.dump(case, f, indent=1)
    print(len(cells))


def score(show: str) -> None:
    """Max relative error per (tail, d1 band, d2) stratum from `rustsheet-eval show --all` output."""
    pts = list(points())
    worst: dict[tuple[str, str, str], tuple[float, str]] = defaultdict(lambda: (0.0, ""))
    text = open(show).read()
    blocks = re.findall(r"^S!([AB])(\d+)\s.*?\n\s+formula : .*?\n\s+expected: (\S+)\n\s+actual  : (\S+)", text, re.M)
    for col, r, exp_s, act_s in blocks:
        exp, act = float(exp_s), float(act_s)
        d1, d2, x = pts[int(r) - 1]
        rel = abs(act - exp) / abs(exp) if exp else abs(act)
        band = "1-2" if d1 <= 2 else ("3-9" if d1 < 10 else "10-20")
        key = ("cdf" if col == "A" else "rt", band, f"{d2:g}")
        if rel >= worst[key][0]:
            worst[key] = (rel, f"{col}{r} ({lit(x)},{d1},{lit(d2)})")
    print(f"cells scored: {len(blocks)}")
    print("| tail | d1 | d2 | max rel. error | at |")
    print("|---|---|---|---|---|")
    for key in sorted(worst, key=lambda k: (k[0], k[1], float(k[2]))):
        rel, at = worst[key]
        print(f"| {key[0]} | {key[1]} | {key[2]} | {rel:.1e} | {at} |")


if __name__ == "__main__":
    {"generate": generate, "score": score}[sys.argv[1]](sys.argv[2])
