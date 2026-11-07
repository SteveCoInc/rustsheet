"""T-4357: write probes-numeric/t4357-poisson-companions, the functions Excel may share a kernel with
POISSON.DIST's cdf below the 1.3a cut, at T-4345's train and hold-out points and at a fresh draw.

Run from the repository root with the tools venv: tools/.venv/bin/python docs/research/t4357/gen_probe.py

Rows: one point (k, m) each. Rows 1:9 are T-4345's train points (probes/t4291 A1:H1, probes/t4302 D19),
rows 10:18 its hold-out (probes-numeric/t4322-poisson-window-r4-fresh A8:C10), rows 19:42 a fresh draw
(seed 4357; k log-uniform in 30..300, m/(k + 1) uniform in [1, 1.3), m to 6 significant digits), fixed
before any candidate was scored. Columns, with a = k + 1:
  A POISSON.DIST(k, m, TRUE)            the cdf itself
  B GAMMA.DIST(m, a, 1, TRUE)           P(a, m): is the cdf 1 - P?
  C 1 - GAMMA.DIST(m, a, 1, TRUE)       the same in Excel's own subtraction
  D GAMMA.DIST(m, a, 1, FALSE)          the gamma density (which prefactor goes with the cdf)
  E POISSON.DIST(k, m, FALSE)           the pmf, the same prefactor up to m / a
  F ERFC.PRECISE(SQRT(a*(m/a - 1 - LN(m/a))))   Temme's erfc argument in Excel arithmetic
"""

import importlib.util
import json
import math
import random
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
_spec = importlib.util.spec_from_file_location("t4345_score", ROOT / "docs/research/t4345/score.py")
assert _spec and _spec.loader
t4345 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t4345)

STEM = "t4357-poisson-companions"
COLUMNS = "ABCDEF"


def fresh(n=24, seed=4357):
    rng = random.Random(seed)
    out = []
    while len(out) < n:
        k = int(round(math.exp(rng.uniform(math.log(30), math.log(300)))))
        m = float(f"{(k + 1) * rng.uniform(1.0, 1.3):.6g}")
        if 1.0 < m / (k + 1) < 1.3 and (k, m) not in out:
            out.append((k, m))
    return out


def points():
    below, _ = t4345.splits()
    known = [(k, m) for split in ("train", "hold-out") for _, k, m, _ in below[split]]
    return known + fresh()


def formulas(k, m):
    a = k + 1
    return [
        f"=POISSON.DIST({k},{m:g},TRUE)",
        f"=GAMMA.DIST({m:g},{a},1,TRUE)",
        f"=1-GAMMA.DIST({m:g},{a},1,TRUE)",
        f"=GAMMA.DIST({m:g},{a},1,FALSE)",
        f"=POISSON.DIST({k},{m:g},FALSE)",
        f"=ERFC.PRECISE(SQRT({a}*({m:g}/{a}-1-LN({m:g}/{a}))))",
    ]


def main():
    pts = points()
    cells = {}
    for row, (k, m) in enumerate(pts, start=1):
        for col, f in zip(COLUMNS, formulas(k, m)):
            cells[f"{col}{row}"] = f
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4357: POISSON.DIST's cdf below 1.3a beside its companions at the same point (k, m), a = k + 1: "
            "A the cdf, B GAMMA.DIST(m,a,1,TRUE), C 1-GAMMA.DIST(m,a,1,TRUE), D GAMMA.DIST(m,a,1,FALSE), "
            "E POISSON.DIST(k,m,FALSE), F ERFC.PRECISE of Temme's argument sqrt(a(m/a-1-LN(m/a))). Rows 1:9 are "
            "T-4345's train points, rows 10:18 its hold-out, rows 19:42 a fresh draw from seed 4357 "
            "(docs/research/t4357/gen_probe.py), all fixed before scoring."
        ),
        "tags": ["M9", "statistical", "probe", "numeric", "T-4357"],
        "expected": {},
        "check": [f"Calc!{c}{r}" for r in range(1, len(pts) + 1) for c in COLUMNS],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    out = ROOT / "eval/suites/probes-numeric/cases" / f"{STEM}.case.json"
    out.write_text(json.dumps(case, indent=2) + "\n")
    print(f"wrote {out.relative_to(ROOT)}: {len(pts)} points")


if __name__ == "__main__":
    main()
