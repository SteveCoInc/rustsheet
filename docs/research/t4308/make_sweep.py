"""T-4308: write probes-numeric/t4308-gamma-density-r4-fresh (run once, after the model was frozen in
commit "GAMMA.DIST's large-shape density is Loader's chain with Excel's bd0 split"; the case is committed).
Decision 0188 R4.2's fresh sweep, seed 43081, literals at 15 significant digits:
- rows 1 to 100: the window, shapes log-uniform in [20, 1E9], x/a log-uniform in [2/3, 3/2], scales in
  [1E-2, 1E2];
- rows 101 to 130: outside the window, shapes in [20, 1E6], x/a in [0.006, 0.66] or [1.52, 3];
- rows 131 to 140: just below the window, x/a in [0.6, 0.667], at shapes outside 30 to 500 (the cdf's
  NEAR_MODE_BELOW range): 20 to 29.9 and 600 to 1E6;
- rows 141 to 150: shapes 1E7 to 1E12 in the window;
- rows 151 to 154: the mode at shapes around the 2 pi a overflow (2E307 below it; 5E307, 9E307 and
  9.99999999999999E307, Excel's largest literal, past it).
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import lit, write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t4308-gamma-density-r4-fresh.case.json"


def loguni(rng: random.Random, lo: float, hi: float) -> float:
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def cell(a: float, u: float, scale: float) -> str:
    a = float(lit(a))
    scale = float(lit(scale))
    return f"=GAMMA.DIST({lit(a * u * scale)},{lit(a)},{lit(scale)},FALSE)"


def main() -> None:
    rng = random.Random(43081)
    cells: dict[str, str] = {}
    for row in range(1, 101):
        cells[f"A{row}"] = cell(loguni(rng, 20, 1e9), loguni(rng, 2 / 3, 1.5), loguni(rng, 1e-2, 1e2))
    for row in range(101, 131):
        u = loguni(rng, 0.006, 0.66) if rng.random() < 0.5 else loguni(rng, 1.52, 3)
        cells[f"A{row}"] = cell(loguni(rng, 20, 1e6), u, loguni(rng, 1e-2, 1e2))
    for row in range(131, 141):
        a = rng.uniform(20, 29.9) if row < 136 else loguni(rng, 600, 1e6)
        cells[f"A{row}"] = cell(a, rng.uniform(0.6, 0.667), 1.0)
    for row in range(141, 151):
        cells[f"A{row}"] = cell(loguni(rng, 1e7, 1e12), loguni(rng, 2 / 3, 1.5), loguni(rng, 1e-2, 1e2))
    for row, a in enumerate((2e307, 5e307, 9e307, 9.99999999999999e307), start=151):
        cells[f"A{row}"] = f"=GAMMA.DIST({lit(a)},{lit(a)},1,FALSE)"
    write_checked_case(
        OUT,
        (
            "T-4308: the R4.2 fresh sweep of GAMMA.DIST's large-shape density (Loader's chain with Excel's bd0 "
            "split), generated after the freeze: the window (shapes 20 to 1E9), outside it, just below it at "
            "shapes outside 30 to 500, shapes 1E7 to 1E12, and the mode around the 2 pi a overflow."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4308"],
        cells,
    )


if __name__ == "__main__":
    main()
