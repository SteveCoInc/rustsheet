"""T-3883 fresh sweep (decision 0188 R4.2), generated after the model was frozen.

Writes eval/suites/probes-numeric/cases/t3883-im-overflow-reciprocal-fresh-sweep.case.json: 60 rows,
|v| log-uniform in [356, 709.7] (where the reciprocal's denominator overflows, up to EXP's own
overflow), the other part w log-uniform in [1e-3, 1e3], both signs random, unfiltered. Columns:
A IMSECH(v+wi), B IMCSCH(v+wi), C IMSEC(w+vi), D IMCSC(w+vi). Seed 3883.
"""

import json
import math
import random
import sys


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi))) * rng.choice((-1.0, 1.0))


def main(out: str) -> None:
    rng = random.Random(3883)
    cells = {}
    for row in range(1, 61):
        v = float("%.10g" % log_uniform(rng, 356.0, 709.7))
        w = float("%.10g" % log_uniform(rng, 1e-3, 1e3))
        cells[f"A{row}"] = f"=IMSECH(COMPLEX({v!r},{w!r}))"
        cells[f"B{row}"] = f"=IMCSCH(COMPLEX({v!r},{w!r}))"
        cells[f"C{row}"] = f"=IMSEC(COMPLEX({w!r},{v!r}))"
        cells[f"D{row}"] = f"=IMCSC(COMPLEX({w!r},{v!r}))"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3883: fresh log-uniform sweep (decision 0188 R4.2), generated after the model was frozen, over the "
            "IM* reciprocals whose denominator overflows. Rows 1-60: |v| in [356, 709.7], the other part w with "
            "|w| in [1e-3, 1e3], signs random, unfiltered; A IMSECH(v+wi), B IMCSCH(v+wi), C IMSEC(w+vi), "
            "D IMCSC(w+vi). Generator: docs/research/t3883/gen_sweep.py (seed 3883)."
        ),
        "tags": ["M9", "engineering", "probe", "T-3883"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    with open(out, "w", encoding="utf-8") as f:
        json.dump(case, f, indent=1)
        f.write("\n")


if __name__ == "__main__":
    main(sys.argv[1])
