"""T-3754: the F tails against 50-digit math on a grid at a small d1 and a huge d2.

`grid.py points > points.txt` writes the grid (d1, d2, x). The engine's tails at those points come from
`grid_dump.rs` (paste it into fisher.rs's test module, run it with T3754_GRID=points.txt and
T3754_OUT=engine.txt, then remove it). `grid.py score engine.txt` prints, per stratum (d1 band, d2
decade), the engine's maximum relative error of F.DIST.RT and F.DIST against mpmath `betainc` at 50
digits, both taken at the exact rational u = d1 x / (d1 x + d2) of the double inputs.

Run from the repo root: tools/.venv/bin/python -I docs/research/t3754/grid.py points|score FILE
"""

from __future__ import annotations

import sys
from collections import defaultdict

import mpmath as mp

mp.mp.dps = 50

D1 = [2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 20]
D2_EXP = [4.31, 4.5, 4.75, 5.0, 5.25, 5.5, 5.75, 6.0, 6.25, 6.5, 6.75, 7.0, 7.5, 8.0, 8.5, 9.0]
XS = [0.5, 0.7, 0.9, 1.0, 1.09080885395379, 1.2, 1.5, 2.0]


def points() -> None:
    for d1 in D1:
        for e in D2_EXP:
            d2 = float(round(10**e)) + 0.0
            for x in XS:
                print(d1, repr(d2), repr(x))
    # The task's own point.
    print(8, repr(5567623.0), repr(1.09080885395379))


def reference(d1: float, d2: float, x: float) -> tuple[mp.mpf, mp.mpf]:
    a, b = mp.mpf(d1) / 2, mp.mpf(d2) / 2
    u = mp.mpf(d1) * mp.mpf(x) / (mp.mpf(d1) * mp.mpf(x) + mp.mpf(d2))
    lower = mp.betainc(a, b, 0, u, regularized=True)
    upper = mp.betainc(b, a, 0, 1 - u, regularized=True)
    return lower, upper


def rel(got: float, ref: mp.mpf) -> float:
    return float(abs(mp.mpf(got) / ref - 1)) if ref != 0 else float(abs(got))


def score(path: str) -> None:
    worst: dict[tuple[str, str], tuple[float, str]] = defaultdict(lambda: (0.0, ""))
    overall = (0.0, "")
    for line in open(path):
        d1s, d2s, xs, ups, lows = line.split()
        d1, d2, x = float(d1s), float(d2s), float(xs)
        lower, upper = reference(d1, d2, x)
        err = max(rel(float(ups), upper), rel(float(lows), lower))
        band = "d1<=10" if d1 <= 10 else "d1>10"
        key = (band, f"1e{int(mp.floor(mp.log10(d2)))}")
        where = f"F({x},{d1s},{d2s})"
        if err > worst[key][0]:
            worst[key] = (err, where)
        if err > overall[0]:
            overall = (err, where)
    for key in sorted(worst):
        err, where = worst[key]
        print(f"{key[0]:7} d2~{key[1]:4}  max rel {err:.2e}  at {where}")
    print(f"overall max rel {overall[0]:.2e} at {overall[1]}")


if __name__ == "__main__":
    if sys.argv[1] == "points":
        points()
    else:
        score(sys.argv[2])
