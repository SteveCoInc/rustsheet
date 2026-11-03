"""T-3192: the fresh a = b = 1/2 sweeps of BETA.DIST's cdf (RESULTS.md).

--sweep 1 writes eval/suites/probes-numeric/cases/t3192-beta-half-half.case.json: x in column A (a constant), then
B =BETA.DIST(x,0.5,0.5,TRUE), C =BETADIST(x,0.5,0.5), D =2*ATAN(SQRT(x)/SQRT(1-x))/PI() (Excel's own worksheet ops).
--sweep 2 writes t3192-beta-half-half-2.case.json (column B only), filled after the 0.9 switch was frozen on sweep 1:
the out-of-sample test of the switch, dense on 0.8-1.
Seeded per sweep, so rerunning reproduces both cases. Usage (repo root):
  python3 docs/research/t3192/make_probe.py --sweep 1|2
"""
import argparse
import json
import math
import random
from pathlib import Path

CASES = Path("eval/suites/probes-numeric/cases")


def ulp_steps(center: float, steps: int) -> list[float]:
    """center and its `steps` nearest doubles on each side, below and above interleaved."""
    xs, lo, hi = [center], center, center
    for _ in range(steps):
        lo, hi = math.nextafter(lo, 0), math.nextafter(hi, 1)
        xs += [lo, hi]
    return xs


def sweep_1() -> list[float]:
    rng = random.Random(3192)
    xs = [10 ** rng.uniform(-300, -5) for _ in range(60)]
    xs += [rng.uniform(1e-5, 0.5) for _ in range(100)]
    xs += ulp_steps(0.5, 20)
    xs += [rng.uniform(0.5, 1) for _ in range(100)]
    xs += [1 - 10 ** rng.uniform(-15, -5) for _ in range(20)]
    return xs


def sweep_2() -> list[float]:
    rng = random.Random(31922)
    xs = [rng.uniform(0.0, 0.8) for _ in range(80)]
    xs += [rng.uniform(0.8, 1.0) for _ in range(150)]
    xs += [rng.uniform(0.88, 0.92) for _ in range(60)]
    xs += [1 - 10 ** rng.uniform(-15, -1) for _ in range(30)]
    return xs + ulp_steps(0.9, 10)


SWEEPS = {
    1: (
        sweep_1,
        "t3192-beta-half-half",
        True,
        "T-3192: BETA.DIST and BETADIST at a = b = 1/2 along x (fresh hold-out for the arcsine closed form), "
        "with Excel's worksheet 2*ATAN(SQRT(x)/SQRT(1-x))/PI() beside them",
    ),
    2: (
        sweep_2,
        "t3192-beta-half-half-2",
        False,
        "T-3192: BETA.DIST at a = b = 1/2 along x, second fresh sweep (out-of-sample test of the x = 0.9 "
        "switch between the direct and the reflected arctangent form)",
    ),
}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sweep", type=int, choices=sorted(SWEEPS), required=True)
    points, name, extra_columns, description = SWEEPS[parser.parse_args().sweep]
    xs = points()
    cells: dict[str, object] = {}
    for i, x in enumerate(xs, start=1):
        cells[f"A{i}"] = x
        cells[f"B{i}"] = f"=BETA.DIST(A{i},0.5,0.5,TRUE)"
        if extra_columns:
            cells[f"C{i}"] = f"=BETADIST(A{i},0.5,0.5)"
            cells[f"D{i}"] = f"=2*ATAN(SQRT(A{i})/SQRT(1-A{i}))/PI()"
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "probe", "numeric", "T-3192", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    out = CASES / f"{name}.case.json"
    out.write_text(json.dumps(case, indent=1) + "\n")
    print(len(xs), "rows ->", out)


main()
