"""T-4296: write probes-numeric/t4296-gamma-density-r4-fresh (run once, after the window chain was
frozen; the case is committed). Decision 0188 R4.2's fresh sweep until GAMMA.DIST has a Gate A
table (T-3925): 160 log-uniform densities from seed 42961. Shapes in [20, 1E6], x/a in the window
[2/3, 3/2] (log-uniform), scales in [1E-2, 1E2]; rows 1 to 40 put x at the mode x = a times the
scale. Literals at 15 significant digits.
"""

import json
import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import lit  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t4296-gamma-density-r4-fresh.case.json"


def loguni(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def main() -> None:
    rng = random.Random(42961)
    cells: dict[str, str] = {}
    for row in range(1, 161):
        a = float(lit(loguni(rng, 20, 1e6)))
        scale = float(lit(loguni(rng, 1e-2, 1e2)))
        u = 1.0 if row <= 40 else loguni(rng, 2 / 3, 1.5)
        cells[f"A{row}"] = f"=GAMMA.DIST({lit(a * u * scale)},{lit(a)},{lit(scale)},FALSE)"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            "T-4296: the R4.2 fresh sweep of the large-shape gamma density's window chain, generated after the freeze: "
            "160 log-uniform densities (shape 20 to 1E6, x/a 2/3 to 3/2, scale 1E-2 to 1E2), rows 1 to 40 at x = a times "
            "the scale."
        ),
        "tags": ["M9", "statistical", "probe", "numeric", "T-4296"],
        "expected": {},
        "check": [f"Calc!A{r}" for r in range(1, 161)],
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    OUT.write_text(json.dumps(case, indent=1) + "\n")


if __name__ == "__main__":
    main()
