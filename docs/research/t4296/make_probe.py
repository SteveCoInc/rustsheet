"""T-4296: write probes/t4296-gamma-density-at-mode (run once; the case is committed).

At x = a the gamma density's exponent E = a ln(x/a) + a - x is exactly 0, so Excel's value reads
out the rest of its window chain: the prefactor and the Stirling correction alone. The power-of-two
shapes of T-4290 (A93, B93, C93) cannot tell the prefactor orders apart, since sqrt(a) is exact.
Column A: the fit shapes, scale 1, dense from 20 to 40 where a short Stirling series would show.
Column B: the hold-out, fixed before any scoring: 40 shapes log-uniform in [20, 1E6] from seed
4296, at 6 significant digits. Column C: the column-A shapes at scale 3 and x = 3a (exact), which
shows whether the scale enters before or after the prefactor.
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import lit, write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes/cases/t4296-gamma-density-at-mode.case.json"

FIT = [
    20.0, 20.5, 21.0, 22.0, 23.5, 25.0, 27.0, 30.0, 32.5, 35.0, 36.0, 40.0, 45.0, 50.0, 60.0, 75.0, 80.0, 81.0,
    99.5, 100.0, 128.5, 150.0, 200.0, 300.0, 499.0, 500.0, 501.0, 700.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0,
    7777.0, 10000.0, 12345.5, 30000.0, 100000.0, 1000000.0, 10000000.0,
]


def holdout() -> list[float]:
    rng = random.Random(4296)
    return [float(f"{math.exp(rng.uniform(math.log(20), math.log(1e6))):.6g}") for _ in range(40)]


def main() -> None:
    cells: dict[str, str] = {}
    for row, (a, h) in enumerate(zip(FIT, holdout(), strict=True), start=1):
        cells[f"A{row}"] = f"=GAMMA.DIST({lit(a)},{lit(a)},1,FALSE)"
        cells[f"B{row}"] = f"=GAMMA.DIST({lit(h)},{lit(h)},1,FALSE)"
        cells[f"C{row}"] = f"=GAMMA.DIST({lit(3 * a)},{lit(a)},3,FALSE)"
    write_checked_case(
        OUT,
        (
            "T-4296: Excel's large-shape gamma density at the mode x = a, where the exponent is 0: fit shapes "
            "20 to 1E7 (A), the hold-out of 40 log-uniform shapes from seed 4296 (B), and the A shapes at "
            "scale 3 (C)."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4296"],
        cells,
    )


if __name__ == "__main__":
    main()
