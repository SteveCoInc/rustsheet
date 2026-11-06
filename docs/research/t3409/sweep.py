"""T-3409 fresh sweep (decision 0188 R4): drawn after the search model's free parameters were frozen
(huge_shape.rs: SEARCH_STEP_LIMIT = 207 at SEARCH_STEP_TOLERANCE = 1E-8, fitted on the 805 cells of
probes/t2982-*, t3070-*, t3350-*, t3396-* and probes-numeric/fit-audit-t3070-*, t3350-r4-*), and
scored against Excel (the oracle sidecar) after the freeze.

    python3 sweep.py generate CASE.json
    python3 sweep.py generate-wide CASE.json   # the second stratum (generate_wide)

Strata (seed below):
- region (120): GAMMA.INV, shape log-uniform over [3E8, 1.3E10], p log-uniform over [1E-300, 0.5]
  or 1 - p log-uniform over [1E-15, 0.5], each side with probability 1/2.
- edge (100): as region, kept only where a rough step count (the fall in ln density from the start
  to the quantile, through the mode if the start lies past it) is within 190 to 225, so the
  points crowd the predicted edge without using the frozen model itself.
- chi (40): CHISQ.INV.RT (20) and CHISQ.INV (20), df log-uniform over [6E8, 2E10], p as region.
- start (20): CHISQ.INV.RT, df log-uniform over [1E6, 1E9], p log-uniform over [1E-300, 1E-250]
  (the start's density underflow cut).
p is written with at most 15 significant digits.
"""

from __future__ import annotations

import json
import math
import random
import sys
from statistics import NormalDist

SEED = 3409
OFFSET = 0.0000886


def literal(x: float) -> str:
    return f"{float(f'{x:.15g}'):.15g}".upper().replace("E-0", "E-").replace("E+", "E")


def draw_p(rng: random.Random) -> float:
    if rng.random() < 0.5:
        return 10.0 ** rng.uniform(-300.0, -0.30103)
    return 1.0 - 10.0 ** rng.uniform(-15.0, -0.30103)


def rough_steps(shape: float, p: float) -> float:
    z = NormalDist().inv_cdf(p) if p < 0.5 else -NormalDist().inv_cdf(1.0 - p)
    zs = z + ((1.0 + OFFSET) ** 3 - 1.0) * math.sqrt(shape)
    return (z * z - zs * zs) / 2.0 if zs < 0.0 else (z * z + zs * zs) / 2.0


def generate(path: str) -> None:
    rng = random.Random(SEED)
    cells: dict[str, str] = {}
    row = 1

    def add(formula: str) -> None:
        nonlocal row
        cells[f"A{row}"] = formula
        row += 1

    def shape_p(lo: float, hi: float) -> tuple[float, str]:
        while True:
            shape = float(f"{10.0 ** rng.uniform(math.log10(lo), math.log10(hi)):.4g}")
            text = literal(draw_p(rng))
            if 0.0 < float(text) < 1.0:
                return shape, text

    for _ in range(120):
        shape, text = shape_p(3e8, 1.3e10)
        add(f"=GAMMA.INV({text},{literal(shape)},1)")
    kept = 0
    while kept < 100:
        shape, text = shape_p(3e8, 1.3e10)
        if 190.0 <= rough_steps(shape, float(text)) <= 225.0:
            add(f"=GAMMA.INV({text},{literal(shape)},1)")
            kept += 1
    for function in ("CHISQ.INV.RT", "CHISQ.INV"):
        for _ in range(20):
            df, text = shape_p(6e8, 2e10)
            df = round(df)
            if function == "CHISQ.INV":
                df = min(df, 10_000_000_000)
            add(f"={function}({text},{df})")
    for _ in range(20):
        df = round(10.0 ** rng.uniform(6.0, 9.0))
        text = literal(10.0 ** rng.uniform(-300.0, -250.0))
        add(f"=CHISQ.INV.RT({text},{df})")
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-3409 fresh sweep (decision 0188 R4): GAMMA.INV, CHISQ.INV and CHISQ.INV.RT at shapes 3E8 to 1.3E10 around Excel's search limit, and CHISQ.INV.RT at a tiny p from df 1E6 (the start cut); drawn after the model was frozen by docs/research/t3409/sweep.py.",
        "tags": ["M9", "probe", "statistics", "T-3409", "sweep", "incomplete-gamma"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    with open(path, "w", encoding="utf-8") as out:
        json.dump(case, out, indent=1)
        out.write("\n")


def main() -> None:
    if len(sys.argv) == 3 and sys.argv[1] == "generate":
        generate(sys.argv[2])
    elif len(sys.argv) == 3 and sys.argv[1] == "generate-wide":
        generate_wide(sys.argv[2])
    else:
        sys.exit(__doc__)


def generate_wide(path: str) -> None:
    """The review's second stratum (drawn after the start rule went back to shape 1E9 and the limit
    stayed 207): the shapes the search model now reaches outside the first sweep, 1E5 to 3E8
    (GAMMA.INV 60, CHISQ.INV.RT 10, CHISQ.INV 10) and 1.3E10 to 1E12 (GAMMA.INV 30, CHISQ.INV.RT 10),
    shape and p drawn as in the first sweep (seed 34092)."""
    rng = random.Random(SEED * 10 + 2)
    cells: dict[str, str] = {}
    row = 1
    for function, count, lo, hi in (
        ("GAMMA.INV", 60, 1e5, 3e8),
        ("CHISQ.INV.RT", 10, 1e5, 3e8),
        ("CHISQ.INV", 10, 1e5, 3e8),
        ("GAMMA.INV", 30, 1.3e10, 1e12),
        ("CHISQ.INV.RT", 10, 1.3e10, 1e12),
    ):
        for _ in range(count):
            while True:
                shape = float(f"{10.0 ** rng.uniform(math.log10(lo), math.log10(hi)):.4g}")
                text = literal(draw_p(rng))
                if 0.0 < float(text) < 1.0:
                    break
            if function == "GAMMA.INV":
                cells[f"A{row}"] = f"=GAMMA.INV({text},{literal(shape)},1)"
            else:
                cells[f"A{row}"] = f"={function}({text},{round(2 * shape)})"
            row += 1
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-3409 fresh sweep, second stratum (decision 0188 R4): GAMMA.INV, CHISQ.INV and CHISQ.INV.RT at shapes 1E5 to 3E8 and 1.3E10 to 1E12, drawn after the model was frozen by docs/research/t3409/sweep.py generate-wide.",
        "tags": ["M9", "probe", "statistics", "T-3409", "sweep", "incomplete-gamma"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    with open(path, "w", encoding="utf-8") as out:
        json.dump(case, out, indent=1)
        out.write("\n")


if __name__ == "__main__":
    main()
