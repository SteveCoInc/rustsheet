"""T-3982: fresh sweep (decision 0188 R4.2) for real IMSECH's cosh, drawn after the rule was frozen.

The frozen rule: a real IMSECH is 1 over IMCOSH's cosh x, libm's below |x| 1E-4 (`LIBM_COSH_BELOW`) and
the `1/u` form's above it. Deterministic (seed 3982). Writes
eval/suites/probes-numeric/cases/t3982-r4-imsech-real-cosh.case.json: 300 rows, |x| log-uniform in
[1E-6, 0.0105] with a random sign (so the gap 1E-4..0.004 between the evidence brackets is dense),
15 significant digits; A IMSECH("x"), B IMCOSH("x"). No x is shared with an older case's literal text.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
NAME = "t3982-r4-imsech-real-cosh"


def sig15(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def main() -> None:
    rng = random.Random(3982)
    lo, hi = math.log(1e-6), math.log(0.0105)
    xs: list[str] = []
    seen: set[str] = set()
    while len(xs) < 300:
        x = sig15(rng.choice((-1, 1)) * math.exp(rng.uniform(lo, hi)))
        if x not in seen:
            seen.add(x)
            xs.append(x)
    cells: dict[str, str] = {}
    for row, x in enumerate(xs, start=1):
        cells[f"A{row}"] = f'=IMSECH("{x}")'
        cells[f"B{row}"] = f'=IMCOSH("{x}")'
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-3982 fresh sweep (decision 0188 R4.2), drawn after the real-IMSECH cosh rule was frozen: "
            "IMSECH (A) and IMCOSH (B) of real x, |x| log-uniform in [1E-6, 0.0105] with a random sign. "
            "Generator: docs/research/t3982/gen_sweep.py (seed 3982)."
        ),
        "tags": ["M9", "engineering", "probe", "T-3982"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    (CASES / f"{NAME}.case.json").write_text(json.dumps(case, indent=2) + "\n")


if __name__ == "__main__":
    main()
