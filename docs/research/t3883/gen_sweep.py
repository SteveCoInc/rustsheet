"""T-3883's fresh sweep (decision 0188 R4.2), written after the kernel choice was frozen.

IMSECH (A) and IMCSCH (B) of h+ti, IMSEC (C) and IMCSC (D) of t+hi, where the hyperbolic argument
|h| is log-uniform in [355, 711] (the band where the reciprocal's denominator overflows) and the
trigonometric argument |t| log-uniform in [1e-3, 1e4], each with a random sign. Seed 3883.
Run: python3 docs/research/t3883/gen_sweep.py > eval/suites/probes-numeric/cases/t3883-overflow-reciprocal-sweep.case.json
"""

import json
import math
import random

ROWS = 50


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def text(re: float, im: float) -> str:
    return f"{re:.15g}{im:+.15g}i"


def main() -> None:
    rng = random.Random(3883)
    cells: dict[str, str] = {}
    for row in range(1, ROWS + 1):
        h = rng.choice((-1, 1)) * float(f"{log_uniform(rng, 355.0, 711.0):.15g}")
        t = rng.choice((-1, 1)) * float(f"{log_uniform(rng, 1e-3, 1e4):.15g}")
        cells[f"A{row}"] = f'=IMSECH("{text(h, t)}")'
        cells[f"B{row}"] = f'=IMCSCH("{text(h, t)}")'
        cells[f"C{row}"] = f'=IMSEC("{text(t, h)}")'
        cells[f"D{row}"] = f'=IMCSC("{text(t, h)}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3883: fresh sweep after the freeze (decision 0188 R4.2) of IMSECH (A) and IMCSCH (B) of h+ti "
            "and IMSEC (C) and IMCSC (D) of t+hi, |h| log-uniform in [355, 711] (the reciprocal's overflowed "
            "denominator), |t| log-uniform in [1e-3, 1e4], random signs. Generator: "
            "docs/research/t3883/gen_sweep.py (seed 3883)."
        ),
        "tags": ["M9", "engineering", "probe", "T-3883"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    print(json.dumps(case, indent=2))


if __name__ == "__main__":
    main()
