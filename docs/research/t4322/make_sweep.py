"""T-4322: write probes-numeric/t4322-poisson-window-r4-fresh (run once, after the model was frozen).

The R4.2 fresh sweep (decision 0188) of the upper-window form of POISSON.DIST's cdf: 48 points from
seed 43220, k log-uniform on [30, 300] (rounded to an integer), mean / (k + 1) uniform on [1.3, 1.5],
the mean cut to 6 significant digits, with no selection (rows 1:6, columns A:H, column-major).
Rows 8:13 of columns A:C are a ladder across the frozen 1.3 cut at k = 100, 200 and 300:
mean / (k + 1) = 1.29, 1.295, 1.298, 1.3, 1.302 and 1.305.
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from sweep_case import write_checked_case  # importable once the path above is set

OUT = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases/t4322-poisson-window-r4-fresh.case.json"
LADDER = (1.29, 1.295, 1.298, 1.3, 1.302, 1.305)


def points(seed=43220):
    """48 points (k, mean): k log-uniform on [30, 300], mean / (k + 1) uniform on [1.3, 1.5], 6 digits."""
    rng = random.Random(seed)
    pts = []
    while len(pts) < 48:
        k = round(math.exp(rng.uniform(math.log(30), math.log(300))))
        mean = float(f"{(k + 1) * rng.uniform(1.3, 1.5):.6g}")
        if 1.3 <= mean / (k + 1) <= 1.5:
            pts.append((k, mean))
    return pts


def main() -> None:
    cells = {}
    for i, (k, mean) in enumerate(points()):
        col, row = "ABCDEFGH"[i // 6], 1 + i % 6
        cells[f"{col}{row}"] = f"=POISSON.DIST({k},{mean:g},TRUE)"
    for col, k in zip("ABC", (100, 200, 300)):
        for j, ratio in enumerate(LADDER):
            cells[f"{col}{8 + j}"] = f"=POISSON.DIST({k},{float(f'{(k + 1) * ratio:.6g}'):g},TRUE)"
    write_checked_case(
        OUT,
        (
            "T-4322: fresh sweep after the freeze (decision 0188 R4.2) of POISSON.DIST's cdf in the upper "
            "gamma window: 48 points from seed 43220, k 30 to 300 log-uniform, mean/(k + 1) uniform in "
            "[1.3, 1.5] (rows 1:6); rows 8:13 of A:C a ladder across the 1.3 cut at k = 100, 200, 300."
        ),
        ["M9", "statistical", "probe", "numeric", "T-4322"],
        cells,
    )


if __name__ == "__main__":
    main()
