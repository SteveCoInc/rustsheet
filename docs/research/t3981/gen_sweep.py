"""T-3981: dense real IMSECH / IMCOSH sweep at 0.004 <= |x| <= 0.01 for the T-3982 fit.

Deterministic. Writes two cases to eval/suites/probes-numeric/cases/:
- t3981-imsech-real-band-sweep (seed 3981): 300 rows, x uniform in [0.004, 0.01] with a random sign,
  15 significant digits. A IMSECH("x"), B IMCOSH("x"), C COSH(x), D 1/COSH(x). Rows 1-21 hold the x of
  t3721 S!C390 (-0.00802240437783925, the one real IMSECH text off Excel) and the 10 neighbouring doubles
  on each side as 17-significant-digit literals (math.nextafter).
- t3981-imsech-real-band-sweep-hold (seed 39811): 150 rows, same rule, no shared x; never used for fitting.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"


def sig15(v: float) -> str:
    return f"{v:.15g}".replace("e", "E")


def write(name: str, xs: list[str], description: str) -> None:
    cells: dict[str, str] = {}
    for row, x in enumerate(xs, start=1):
        cells[f"A{row}"] = f'=IMSECH("{x}")'
        cells[f"B{row}"] = f'=IMCOSH("{x}")'
        cells[f"C{row}"] = f"=COSH({x})"
        cells[f"D{row}"] = f"=1/COSH({x})"
    case = {
        "format": "rustsheet-case/v1",
        "description": description,
        "tags": ["M9", "engineering", "probe", "T-3981"],
        "expected": {},
        "sheets": [{"name": "S", "cells": cells}],
    }
    (CASES / f"{name}.case.json").write_text(json.dumps(case, indent=2) + "\n")


def draw(rng: random.Random, n: int, seen: set[str]) -> list[str]:
    out: list[str] = []
    while len(out) < n:
        x = sig15(rng.choice((-1, 1)) * rng.uniform(0.004, 0.01))
        if x not in seen:
            seen.add(x)
            out.append(x)
    return out


def main() -> None:
    seen: set[str] = set()
    x0 = float("-0.00802240437783925")
    near = [repr(x0)]
    lo = hi = x0
    for _ in range(10):  # the 10 neighbouring doubles each side, 17 significant digits
        lo, hi = math.nextafter(lo, -1.0), math.nextafter(hi, 1.0)
        near = [repr(lo)] + near + [repr(hi)]
    xs = near  # 21 literals: C390 plus 10 each side
    seen.update(xs)
    xs += draw(random.Random(3981), 300 - len(xs), seen)
    write(
        "t3981-imsech-real-band-sweep",
        xs,
        "T-3981: IMSECH (A), IMCOSH (B), worksheet COSH (C) and 1/COSH (D) of real x, 0.004 <= |x| <= 0.01. Rows "
        "1-21 hold t3721 S!C390's x and its 10 neighbouring doubles each side as 17-digit literals; "
        "rows 22-300 x uniform with a random sign. Generator: docs/research/t3981/gen_sweep.py (seed 3981).",
    )
    write(
        "t3981-imsech-real-band-sweep-hold",
        draw(random.Random(39811), 150, seen),
        "T-3981: hold-out for T-3982, never used for fitting (decision 0188 R4.1): same rule as "
        "t3981-imsech-real-band-sweep, 150 rows, no shared x. Generator: docs/research/t3981/gen_sweep.py "
        "(seed 39811).",
    )


if __name__ == "__main__":
    main()
