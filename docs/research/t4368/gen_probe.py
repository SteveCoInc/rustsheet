"""T-4368: write probes/t4368-poisson-shape-and-edge, the probes that separate where POISSON.DIST's cdf below
1.3a carries Excel's error.

Run from the repository root with the tools venv: tools/.venv/bin/python docs/research/t4368/gen_probe.py

Part 1, rows 1:42: T-4357's points (rows 1:9 T-4345 train, 10:18 its hold-out, 19:42 T-4357's fresh draw, seed
4357), a = k + 1. Is the cdf's error the gamma upper tail's at any shape, or special to integer shapes?
  A POISSON.DIST(k, m, TRUE)          the cdf (Q(a, m))
  B CHISQ.DIST.RT(2m, 2a)             Q(a, m) through the chi-squared code
  C CHISQ.DIST.RT(2m, 2a - 1)         Q(a - 1/2, m), a half-integer shape
  D CHISQ.DIST.RT(2m, 2a + 1)         Q(a + 1/2, m)
  E GAMMA.DIST(m, a + 0.5, 1, TRUE)   P(a + 1/2, m), the non-integer lower tail
Part 2, rows 43 on: m stepped across 1.0a, where a series / continued-fraction switch at x = a would show:
k in 50, 100, 200, 300 and m = a (1 + j / 1000) for j = -4 .. 8, m to 7 significant digits. Columns A:E as above.
Every point was fixed before any candidate was scored.
"""

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
_spec = importlib.util.spec_from_file_location("t4357_gen", ROOT / "docs/research/t4357/gen_probe.py")
assert _spec and _spec.loader
t4357 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4357)

STEM = "t4368-poisson-shape-and-edge"
COLUMNS = "ABCDE"
EDGE_K = (50, 100, 200, 300)
EDGE_J = range(-4, 9)


def edge_points():
    return [(k, float(f"{(k + 1) * (1 + j / 1000):.7g}")) for k in EDGE_K for j in EDGE_J]


def points():
    return t4357.points() + edge_points()


def formulas(k, m):
    a = k + 1
    return [
        f"=POISSON.DIST({k},{m:.15g},TRUE)",
        f"=CHISQ.DIST.RT({2 * m:.15g},{2 * a})",
        f"=CHISQ.DIST.RT({2 * m:.15g},{2 * a - 1})",
        f"=CHISQ.DIST.RT({2 * m:.15g},{2 * a + 1})",
        f"=GAMMA.DIST({m:.15g},{a}.5,1,TRUE)",
    ]


def main():
    pts = points()
    cells = {f"{c}{r}": f for r, (k, m) in enumerate(pts, start=1) for c, f in zip(COLUMNS, formulas(k, m))}
    description = (
        "T-4368: POISSON.DIST's cdf below 1.3a beside the gamma upper tail at the same and at half-integer "
        "shapes, a = k + 1: A POISSON.DIST(k,m,TRUE), B CHISQ.DIST.RT(2m,2a), C CHISQ.DIST.RT(2m,2a-1), "
        "D CHISQ.DIST.RT(2m,2a+1), E GAMMA.DIST(m,a+0.5,1,TRUE). Rows 1:42 are T-4357's points (train 1:9, "
        "hold-out 10:18, fresh 19:42); rows 43:94 step m across 1.0a for k 50, 100, 200, 300 "
        "(docs/research/t4368/gen_probe.py), all fixed before scoring."
    )
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "statistical", "probe", "numeric", "T-4368"],
        "expected": {},
        "check": [f"Calc!{c}{r}" for r in range(1, len(pts) + 1) for c in COLUMNS],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    out = ROOT / "eval/suites/probes/cases" / f"{STEM}.case.json"
    out.write_text(json.dumps(case, indent=2) + "\n")
    print(f"wrote {out.relative_to(ROOT)}: {len(pts)} points")

if __name__ == "__main__":
    main()
