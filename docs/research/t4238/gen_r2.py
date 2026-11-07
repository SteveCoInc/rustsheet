"""Generate probes/t4238-r2-absorb-orientation (T-4238 round 2): which shape's absorption is Excel's #NUM!.

Gate A showed the first shape absorbed by the second is a value (accuracy/BETA.DIST/s1 A183, A190), while the
second absorbed by the first is #NUM! (probes/t4238-negbinom-bfrac-huge-s). Fresh points, predictions committed
before the fill: H_oriented: #NUM! on bfrac where fl(a0 + b0) = a0 after bratio's swap (density: fl(alpha + beta) =
alpha, both above 1); H_raw: the same test on the raw arguments (alpha, beta), no swap.
Run: python3 docs/research/t4238/gen_r2.py > eval/suites/probes/cases/t4238-r2-absorb-orientation.case.json
"""

import json
import sys

# (x, alpha, beta) points, one per row: column A TRUE, column B FALSE.
POINTS = """
0.676787845512177 41 6E+17 | 0.676787845512177 41 5E+17 | 0.676787845512177 101 2E+18 | 0.676787845512177 101 1E+18
0.676787845512177 1001 1E+19 | 0.676787845512177 1001 9E+18 | 0.1 41 6E+17 | 0.9 41 6E+17 | 0.5 41 6E+17
0.9 6E+17 41 | 0.5 6E+17 41 | 0.999 7E+17 57.5 | 0.001 57.5 7E+17 | 1E-19 41 6E+17 | 0.25 3E+20 2.5 | 0.75 2.5 3E+20
"""
CELLS = [tuple(p.split()) for line in POINTS.strip().splitlines() for p in line.split("|")]


def main() -> None:
    cells: dict[str, str] = {}
    for i, (x, a, b) in enumerate(CELLS, start=1):
        cells[f"A{i}"] = f"=BETA.DIST({x},{a},{b},TRUE)"
        cells[f"B{i}"] = f"=BETA.DIST({x},{a},{b},FALSE)"
    for i, (f, s) in enumerate([("6E+17", "40"), ("40", "6E+17"), ("2E+18", "100.5")], start=len(CELLS) + 1):
        cells[f"A{i}"] = f"=NEGBINOM.DIST({f},{s},0.323212154487823,TRUE)"
        cells[f"B{i}"] = f"=NEGBINOM.DIST({f},{s},0.676787845512177,TRUE)"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4238 round 2: which shape's absorption makes BETA.DIST's and NEGBINOM.DIST's cdf (and BETA.DIST's "
            "density) #NUM!. Predictions in docs/research/t4238/predictions-r2.tsv committed before the fill. "
            "H_oriented: #NUM! on bfrac where fl(a0 + b0) = a0 after bratio's swap (density: fl(alpha + beta) = "
            "alpha, both shapes above 1); H_raw: the same on the raw arguments."
        ),
        "tags": ["M9", "statistical", "beta", "probe"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    sys.stdout.write(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
