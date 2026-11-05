"""T-1749 fresh sweep (decision 0188 R4.2), generated after DEVSQ's two-pass form was frozen.

200 DEVSQ calls on 2 to 12 literal numbers: an offset log-uniform in [1e-3, 1e15] of either sign plus a spread
log-uniform in [1e-6, 1e3] times a uniform draw, so cancellation (spread far below the offset) and plain data both
occur. Literals have 15 significant digits (Excel reads a typed literal to 15, T-4077). Seeded.
Usage: python3 gen_fresh.py OUT.case.json
"""

import json
import math
import random
import sys


def lu(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def main() -> None:
    rng = random.Random(1749)
    cells: dict[str, str] = {}
    for row in range(1, 201):
        offset = lu(rng, 1e-3, 1e15) * rng.choice((-1, 1))
        spread = lu(rng, 1e-6, 1e3)
        n = rng.randint(2, 12)
        xs = [f"{offset + spread * rng.uniform(-1, 1):.15g}" for _ in range(n)]
        cells[f"A{row}"] = f"=DEVSQ({','.join(xs)})"
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-1749 fresh sweep (R4.2, after the freeze): DEVSQ of literal data, with and without cancellation.",
        "tags": ["M9", "probe", "statistics", "T-1749", "sweep", "r4-fresh"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    with open(sys.argv[1], "w", encoding="utf-8") as out:
        json.dump(case, out, indent=1)
        out.write("\n")


if __name__ == "__main__":
    main()
